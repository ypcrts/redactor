//! Test fixtures and PDF builders.
//!
//! Provides builders for creating test PDFs with specific content,
//! following the Builder pattern for clean test setup.

use anyhow::Result;
use printpdf::*;
use std::fs;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

/// Builder for creating test PDFs with custom content.
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// # use anyhow::Result;
/// # fn main() -> Result<()> {
/// let pdf = TestPdfBuilder::new()
///     .with_title("Test Document")
///     .with_verizon_account("123456789-00001")
///     .with_phone("(555) 234-5678")
///     .with_phone("555-987-6543")
///     .with_content("Additional custom content")
///     .build(Path::new("/tmp/test.pdf"))?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct TestPdfBuilder {
    title: String,
    account_numbers: Vec<String>,
    phone_numbers: Vec<String>,
    custom_content: Vec<String>,
    page_width: Mm,
    page_height: Mm,
}

impl TestPdfBuilder {
    /// Creates a new test PDF builder with default settings.
    pub fn new() -> Self {
        Self {
            title: "Test Document".to_string(),
            account_numbers: Vec::new(),
            phone_numbers: Vec::new(),
            custom_content: Vec::new(),
            page_width: Mm(210.0),  // A4 width
            page_height: Mm(297.0), // A4 height
        }
    }
    
    /// Sets the document title.
    pub fn with_title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }
    
    /// Adds a Verizon account number to the document.
    pub fn with_verizon_account(mut self, account: &str) -> Self {
        self.account_numbers.push(account.to_string());
        self
    }
    
    /// Adds a phone number to the document.
    pub fn with_phone(mut self, phone: &str) -> Self {
        self.phone_numbers.push(phone.to_string());
        self
    }
    
    /// Adds custom text content to the document.
    pub fn with_content(mut self, content: &str) -> Self {
        self.custom_content.push(content.to_string());
        self
    }
    
    /// Sets custom page dimensions.
    pub fn with_dimensions(mut self, width: f32, height: f32) -> Self {
        self.page_width = Mm(width);
        self.page_height = Mm(height);
        self
    }
    
    /// Builds the PDF and writes it to the specified path.
    pub fn build(self, output_path: &Path) -> Result<PathBuf> {
        let (doc, page1, layer1) = PdfDocument::new(
            &self.title,
            self.page_width,
            self.page_height,
            "Layer 1"
        );
        let current_layer = doc.get_page(page1).get_layer(layer1);
        
        // Build content string
        let mut content = String::new();
        content.push_str(&format!("{}\n\n", self.title));
        
        // Add account numbers
        if !self.account_numbers.is_empty() {
            content.push_str("Account Information:\n");
            for (i, account) in self.account_numbers.iter().enumerate() {
                content.push_str(&format!("  Account #{}: {}\n", i + 1, account));
            }
            content.push('\n');
        }
        
        // Add phone numbers
        if !self.phone_numbers.is_empty() {
            content.push_str("Contact Information:\n");
            for (i, phone) in self.phone_numbers.iter().enumerate() {
                content.push_str(&format!("  Phone #{}: {}\n", i + 1, phone));
            }
            content.push('\n');
        }
        
        // Add custom content
        for custom in &self.custom_content {
            content.push_str(custom);
            content.push('\n');
        }
        
        // Add text to PDF
        let font = doc.add_builtin_font(BuiltinFont::Helvetica)?;
        current_layer.use_text(&content, 12.0, Mm(20.0), Mm(270.0), &font);
        
        // Save PDF
        doc.save(&mut BufWriter::new(fs::File::create(output_path)?))?;
        
        Ok(output_path.to_path_buf())
    }
}

impl Default for TestPdfBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Quick helper to create a Verizon bill PDF with standard content.
pub fn create_verizon_bill(
    path: &Path,
    account: &str,
    phones: &[&str],
) -> Result<PathBuf> {
    let mut builder = TestPdfBuilder::new()
        .with_title("VERIZON WIRELESS - Monthly Statement")
        .with_verizon_account(account)
        .with_content("Billing Period: January 1-31, 2026")
        .with_content("Payment Due: February 15, 2026")
        .with_content("\nService Summary:");
    
    for phone in phones {
        builder = builder.with_phone(phone);
    }
    
    builder.build(path)
}

/// Quick helper to create a contact list PDF.
pub fn create_contact_list(
    path: &Path,
    contacts: &[(&str, &str)], // (name, phone) pairs
) -> Result<PathBuf> {
    let mut builder = TestPdfBuilder::new()
        .with_title("Contact List")
        .with_content("Emergency Contacts:\n");
    
    for (name, phone) in contacts {
        builder = builder.with_content(&format!("  {}: {}", name, phone));
        builder = builder.with_phone(phone);
    }
    
    builder.build(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_builder_pattern() {
        let builder = TestPdfBuilder::new()
            .with_title("Test")
            .with_verizon_account("123456789-00001")
            .with_phone("(555) 234-5678");
        
        assert_eq!(builder.title, "Test");
        assert_eq!(builder.account_numbers.len(), 1);
        assert_eq!(builder.phone_numbers.len(), 1);
    }
    
    #[test]
    fn test_create_verizon_bill() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let pdf_path = temp_dir.path().join("test.pdf");
        
        create_verizon_bill(
            &pdf_path,
            "123456789-00001",
            &["(555) 234-5678", "555-987-6543"],
        )?;
        
        assert!(pdf_path.exists());
        Ok(())
    }
}
