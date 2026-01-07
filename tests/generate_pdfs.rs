// Run this once to generate the test PDF files
// cargo test --test generate_pdfs -- --nocapture

use anyhow::Result;
use printpdf::*;
use redactor::redactor;
use std::fs;
use std::io::BufWriter;
use std::path::PathBuf;

// This test generates the expected output PDF file
// Run it once: cargo test --test generate_pdfs

fn create_test_pdf_with_single_phone(path: &PathBuf) -> Result<()> {
    let (doc, page1, layer1) = PdfDocument::new("Test Document", Mm(210.0), Mm(297.0), "Layer 1");
    let current_layer = doc.get_page(page1).get_layer(layer1);

    let text_content = r#"
Employee Information Form

Full Name: Sarah Johnson
Employee ID: EMP-2024-789
Department: Engineering
Position: Senior Software Engineer
Date of Hire: March 15, 2020

Contact Details:
Phone Number: (415) 555-1234
Email Address: sarah.johnson@company.com
Office Location: Building B, Room 304

Emergency Contact:
Name: Michael Johnson
Relationship: Spouse

Notes:
Sarah has been a valuable team member for over 4 years.
She specializes in backend systems and database optimization.
"#;

    let font = doc.add_builtin_font(BuiltinFont::Helvetica)?;
    current_layer.use_text(text_content, 12.0, Mm(20.0), Mm(270.0), &font);

    doc.save(&mut BufWriter::new(fs::File::create(path)?))?;
    Ok(())
}

#[test]
fn generate_test_pdfs() -> Result<()> {
    let tests_dir = if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        PathBuf::from(manifest_dir).join("tests")
    } else {
        std::env::current_dir()?.join("tests")
    };

    // Create input PDF
    let input_pdf = tests_dir.join("test_input_single_phone.pdf");
    create_test_pdf_with_single_phone(&input_pdf)?;
    println!("Created input PDF: {}", input_pdf.display());

    // Create expected output PDF by redacting the input
    let output_pdf = tests_dir.join("expected_output_single_phone.pdf");
    redactor::redact_phone_numbers_in_pdf(&input_pdf, &output_pdf)?;
    println!("Created expected output PDF: {}", output_pdf.display());

    Ok(())
}

fn create_test_pdf_type3_encoded_with_phones(path: &PathBuf) -> Result<()> {
    let (doc, page1, layer1) =
        PdfDocument::new("Monthly Statement", Mm(210.0), Mm(297.0), "Layer 1");
    let current_layer = doc.get_page(page1).get_layer(layer1);

    // Simulate a bill document with Type 3 encoded-like text
    // Type 3 fonts often have custom encodings, so we'll use text that looks like
    // what might appear in a scanned or auto-generated bill
    let text_content = r#"
MONTHLY STATEMENT

Account Summary
================

Customer: Jane Smith
Account ID: 12345-ABC-789
Billing Period: Current Month

Service Details:
-----------------
Internet Service (Fiber 1000)
Phone Service
Cloud Storage (100GB)

Contact Information:
--------------------
Primary Phone: (555) 867-5309
Secondary Phone: 555-234-5678
Email: jane.smith@example.com

Charges:
--------
Internet Service:     $89.99
Phone Service:        $45.00
Cloud Storage:        $9.99
Tax:                  $14.50
                      -------
Total Amount Due:    $159.48

Payment Due Date: Next Month

For customer support, call (555) 234-5678
or visit our website at www.provider.com

Emergency Contact: (555) 999-8888
"#;

    let font = doc.add_builtin_font(BuiltinFont::Courier)?;
    current_layer.use_text(text_content, 10.0, Mm(15.0), Mm(280.0), &font);

    doc.save(&mut BufWriter::new(fs::File::create(path)?))?;
    Ok(())
}

#[test]
fn generate_type3_encoded_test_pdfs() -> Result<()> {
    let tests_dir = if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        PathBuf::from(manifest_dir).join("tests")
    } else {
        std::env::current_dir()?.join("tests")
    };

    // Create input PDF with multiple phone numbers (simulating Type 3 encoded text)
    let input_pdf = tests_dir.join("test_input_my_bill.pdf");
    create_test_pdf_type3_encoded_with_phones(&input_pdf)?;
    println!("Created input PDF: {}", input_pdf.display());

    // Create expected output PDF by redacting the input
    let output_pdf = tests_dir.join("expected_output_my_bill.pdf");
    redactor::redact_phone_numbers_in_pdf(&input_pdf, &output_pdf)?;
    println!("Created expected output PDF: {}", output_pdf.display());

    Ok(())
}
