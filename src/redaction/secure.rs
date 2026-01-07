//! Secure redaction strategy using MuPDF.
//!
//! This strategy physically removes text from PDF documents using MuPDF's
//! redaction API, ensuring that redacted content cannot be recovered.

use super::strategy::{RedactionResult, RedactionStrategy, RedactionTarget};
use crate::domain::{
    PatternMatcher, PhoneNumberMatcher, VerizonAccountMatcher, VerizonCallDetailsMatcher,
};
use crate::error::{RedactorError, RedactorResult};
use std::path::Path;

use mupdf::pdf::{PdfAnnotationType, PdfDocument, PdfPage};
use mupdf::Rect as MuRect;

/// Secure redaction strategy that physically removes text using MuPDF.
///
/// This strategy:
/// 1. Creates PDF redaction annotations at target locations
/// 2. Applies redactions using `pdf_redact_page` (physical removal)
/// 3. Saves the modified PDF
///
/// **Security**: Redacted text is completely removed and cannot be extracted.
#[derive(Debug, Clone, Default)]
pub struct SecureRedactionStrategy {
    /// Maximum search hits per pattern (prevents performance issues)
    max_hits: u32,
}

impl SecureRedactionStrategy {
    /// Creates a new secure redaction strategy with default settings.
    pub fn new() -> Self {
        Self { max_hits: 100 }
    }

    /// Sets the maximum number of search hits per pattern.
    pub fn with_max_hits(mut self, max_hits: u32) -> Self {
        self.max_hits = max_hits;
        self
    }

    /// Resolves patterns from redaction targets.
    fn resolve_patterns(
        &self,
        input: &Path,
        targets: &[RedactionTarget],
    ) -> RedactorResult<Vec<String>> {
        let mut patterns = Vec::new();

        for target in targets {
            match target {
                RedactionTarget::Literal(text) => {
                    patterns.push(text.clone());
                }
                RedactionTarget::PhoneNumbers => {
                    let text = self.extract_text(input)?;
                    let matcher = PhoneNumberMatcher::new();
                    for phone_str in matcher.extract_all(&text) {
                        if let Some(normalized) = matcher.normalize(phone_str) {
                            patterns.extend(matcher.generate_variants(&normalized));
                        }
                    }
                }
                RedactionTarget::VerizonAccount => {
                    let text = self.extract_text(input)?;
                    if let Some(account) = VerizonAccountMatcher::find_account_number(&text) {
                        let matcher = VerizonAccountMatcher::new();
                        patterns.extend(matcher.generate_variants(&account));
                    } else {
                        return Err(RedactorError::PatternNotFound {
                            pattern: "Verizon account number".to_string(),
                            context: "document text".to_string(),
                        });
                    }
                }
                RedactionTarget::VerizonCallDetails => {
                    let text = self.extract_text(input)?;
                    let matcher = VerizonCallDetailsMatcher::new();

                    // Check if document contains call detail table
                    if VerizonCallDetailsMatcher::has_call_detail_table(&text) {
                        // Extract all call detail column values (time, origination, destination)
                        let details = matcher.extract_all_call_details(&text);
                        patterns.extend(details);
                    }
                    // Note: If no call detail table found, we simply don't add patterns
                    // This is not an error - the document may not have call details
                }
                RedactionTarget::Regex(_pattern) => {
                    // Regex-based search would require a different approach
                    // For now, return an error indicating unsupported
                    return Err(RedactorError::InvalidInput {
                        parameter: "target".to_string(),
                        reason: "Regex targets not yet supported in secure strategy".to_string(),
                    });
                }
                RedactionTarget::AllText => {
                    // For AllText, we'll redact the entire page by searching for a pattern that matches everything
                    // Use a regex that matches any character sequence
                    patterns.push(".+".to_string());
                }
            }
        }

        // Return empty patterns vector if none found - this will result in
        // zero redactions but is not an error condition
        Ok(patterns)
    }

    /// Applies redactions to a PDF using MuPDF.
    fn apply_mupdf_redactions(
        &self,
        pdf_doc: &PdfDocument,
        patterns: &[String],
    ) -> RedactorResult<RedactionResult> {
        let page_count = pdf_doc
            .page_count()
            .map_err(|e| RedactorError::BackendError {
                backend: "MuPDF".to_string(),
                message: format!("Failed to get page count: {}", e),
                source: Some(Box::new(e)),
            })?;

        let mut result = RedactionResult {
            pages_processed: page_count as usize,
            secure: true,
            ..Default::default()
        };

        // Check if this is an "all text" redaction (pattern is ".+")
        let redact_all = patterns.len() == 1 && patterns[0] == ".+";

        // Process each page
        for page_idx in 0..page_count {
            let page = pdf_doc
                .load_page(page_idx)
                .map_err(|e| RedactorError::PdfProcessing {
                    message: format!("Failed to load page {}", page_idx + 1),
                    page: Some(page_idx as usize + 1),
                    source: Some(Box::new(e)),
                })?;

            // Convert to PDF page for annotation support
            let mut pdf_page = match PdfPage::try_from(page.clone()) {
                Ok(p) => p,
                Err(_) => continue, // Skip non-PDF pages
            };

            let mut page_redactions = 0;

            // If redacting all, create a single annotation covering the entire page
            if redact_all {
                let bounds = page.bounds().map_err(|e| RedactorError::BackendError {
                    backend: "MuPDF".to_string(),
                    message: format!("Failed to get bounds for page {}", page_idx + 1),
                    source: Some(Box::new(e)),
                })?;

                let annot = pdf_page
                    .create_annotation(PdfAnnotationType::Redact)
                    .map_err(|e| RedactorError::PdfProcessing {
                        message: "Failed to create redaction annotation".to_string(),
                        page: Some(page_idx as usize + 1),
                        source: Some(Box::new(e)),
                    })?;

                unsafe {
                    ffi::set_annotation_rect(&annot, bounds);
                }

                page_redactions += 1;
            } else {
                // Search for each pattern
                for pattern in patterns {
                    let hits = page.search(pattern, self.max_hits).map_err(|e| {
                        RedactorError::BackendError {
                            backend: "MuPDF".to_string(),
                            message: format!("Search failed for pattern: {}", pattern),
                            source: Some(Box::new(e)),
                        }
                    })?;

                    // Create redaction annotation for each hit
                    for quad in hits {
                        let annot = pdf_page
                            .create_annotation(PdfAnnotationType::Redact)
                            .map_err(|e| RedactorError::PdfProcessing {
                                message: "Failed to create redaction annotation".to_string(),
                                page: Some(page_idx as usize + 1),
                                source: Some(Box::new(e)),
                            })?;

                        // Calculate bounding rectangle
                        let rect = MuRect {
                            x0: quad.ul.x.min(quad.ll.x).min(quad.ur.x).min(quad.lr.x),
                            y0: quad.ul.y.min(quad.ll.y).min(quad.ur.y).min(quad.lr.y),
                            x1: quad.ul.x.max(quad.ll.x).max(quad.ur.x).max(quad.lr.x),
                            y1: quad.ul.y.max(quad.ll.y).max(quad.ur.y).max(quad.lr.y),
                        };

                        // Set annotation rectangle
                        unsafe {
                            ffi::set_annotation_rect(&annot, rect);
                        }

                        page_redactions += 1;
                    }
                }
            } // end else block for pattern-based redaction

            // Apply redactions if any were created
            if page_redactions > 0 {
                pdf_page
                    .redact()
                    .map_err(|e| RedactorError::PdfProcessing {
                        message: format!("Failed to apply redactions on page {}", page_idx + 1),
                        page: Some(page_idx as usize + 1),
                        source: Some(Box::new(e)),
                    })?;

                result.instances_redacted += page_redactions;
                result.pages_modified += 1;
            }
        }

        Ok(result)
    }
}

impl RedactionStrategy for SecureRedactionStrategy {
    fn redact(
        &self,
        input: &Path,
        output: &Path,
        targets: &[RedactionTarget],
    ) -> RedactorResult<RedactionResult> {
        // Resolve patterns from targets
        let patterns = self.resolve_patterns(input, targets)?;

        // If no patterns found, just copy the file
        if patterns.is_empty() {
            std::fs::copy(input, output).map_err(|e| RedactorError::Io {
                path: output.to_path_buf(),
                source: e,
            })?;
            return Ok(RedactionResult::none());
        }

        // Open PDF with MuPDF
        let input_str = input.to_str().ok_or_else(|| RedactorError::InvalidInput {
            parameter: "input".to_string(),
            reason: "Path contains invalid UTF-8".to_string(),
        })?;

        let pdf_doc = PdfDocument::open(input_str).map_err(|e| RedactorError::PdfProcessing {
            message: "Failed to open PDF with MuPDF".to_string(),
            page: None,
            source: Some(Box::new(e)),
        })?;

        // Apply redactions
        let result = self.apply_mupdf_redactions(&pdf_doc, &patterns)?;

        // Save if redactions were applied
        if result.has_redactions() {
            let output_str = output.to_str().ok_or_else(|| RedactorError::InvalidInput {
                parameter: "output".to_string(),
                reason: "Path contains invalid UTF-8".to_string(),
            })?;

            pdf_doc
                .save(output_str)
                .map_err(|e| RedactorError::PdfProcessing {
                    message: "Failed to save redacted PDF".to_string(),
                    page: None,
                    source: Some(Box::new(e)),
                })?;
        } else {
            // No redactions - just copy the file
            std::fs::copy(input, output).map_err(|e| RedactorError::Io {
                path: output.to_path_buf(),
                source: e,
            })?;
        }

        Ok(result)
    }

    fn extract_text(&self, input: &Path) -> RedactorResult<String> {
        let bytes = std::fs::read(input).map_err(|e| RedactorError::Io {
            path: input.to_path_buf(),
            source: e,
        })?;

        pdf_extract::extract_text_from_mem(&bytes).map_err(|e| RedactorError::TextExtraction {
            path: input.to_path_buf(),
            reason: e.to_string(),
        })
    }

    fn name(&self) -> &str {
        "SecureRedaction"
    }

    fn is_secure(&self) -> bool {
        true
    }
}

/// FFI helpers for MuPDF annotation operations.
mod ffi {
    use mupdf::pdf::PdfAnnotation;
    use mupdf::Rect;

    /// Sets the rectangle for a PDF annotation via FFI.
    ///
    /// # Safety
    /// This function uses unsafe FFI calls to access MuPDF's C API.
    /// The annotation must be valid and the context properly initialized.
    pub unsafe fn set_annotation_rect(annot: &PdfAnnotation, rect: Rect) {
        #[repr(C)]
        struct PdfAnnotRaw {
            inner: *mut mupdf_sys::pdf_annot,
        }

        let annot_raw = std::mem::transmute::<&PdfAnnotation, &PdfAnnotRaw>(annot);
        let ctx = mupdf_sys::mupdf_new_base_context();

        if !ctx.is_null() {
            let fz_rect = mupdf_sys::fz_rect {
                x0: rect.x0,
                y0: rect.y0,
                x1: rect.x1,
                y1: rect.y1,
            };

            mupdf_sys::pdf_set_annot_rect(ctx, annot_raw.inner, fz_rect);
            mupdf_sys::mupdf_drop_base_context(ctx);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_creation() {
        let strategy = SecureRedactionStrategy::new();
        assert_eq!(strategy.name(), "SecureRedaction");
        assert!(strategy.is_secure());
    }

    #[test]
    fn test_max_hits_configuration() {
        let strategy = SecureRedactionStrategy::new().with_max_hits(50);
        assert_eq!(strategy.max_hits, 50);
    }
}
