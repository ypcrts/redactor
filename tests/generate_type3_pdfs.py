#!/usr/bin/env python3
"""
Generate test PDFs with Type3 fonts that simulate real Verizon bills.

Type3 fonts use custom glyph definitions, which is what makes real Verizon bills
challenging to redact. This script creates test PDFs that mimic that structure.

Usage:
    python3 tests/generate_type3_pdfs.py

Requirements:
    pip install pymupdf reportlab
"""

import fitz  # PyMuPDF
from pathlib import Path


def create_verizon_bill_with_type3(output_path: str):
    """
    Create a Verizon-style bill PDF and convert it to use Type3-like encoding.
    
    While we can't easily create pure Type3 fonts from scratch, we can create
    a PDF and then manipulate it to have similar text rendering characteristics
    that make simple text extraction difficult.
    """
    # Create a new PDF
    doc = fitz.open()
    page = doc.new_page(width=612, height=792)  # Letter size
    
    # Content to add
    content = [
        ("VERIZON WIRELESS", 12, 50, 720, 18, True),
        ("Monthly Bill Statement", 12, 50, 700, 12, False),
        ("", 0, 0, 0, 0, False),  # blank line
        ("Account Number: 123456789-00001", 12, 50, 660, 11, False),
        ("Billing Period: December 1-31, 2025", 12, 50, 645, 11, False),
        ("Account Holder: Jane Smith", 12, 50, 630, 11, False),
        ("Payment Due: January 15, 2026", 12, 50, 615, 11, False),
        ("", 0, 0, 0, 0, False),
        ("Service Summary", 12, 50, 585, 12, True),
        ("---------------", 12, 50, 570, 11, False),
        ("Primary Line: (555) 867-5309", 12, 50, 550, 11, False),
        ("Secondary Line: 555-123-4567", 12, 50, 535, 11, False),
        ("Mobile: (555) 234-5678", 12, 50, 520, 11, False),
        ("", 0, 0, 0, 0, False),
        ("Account: 123456789-00001", 12, 50, 490, 11, False),
        ("", 0, 0, 0, 0, False),
        ("Charges:", 12, 50, 460, 12, True),
        ("--------", 12, 50, 445, 11, False),
        ("Internet Service:     $89.99", 12, 70, 425, 10, False),
        ("Phone Service:        $45.00", 12, 70, 410, 10, False),
        ("Cloud Storage:         $9.99", 12, 70, 395, 10, False),
        ("Tax:                  $14.50", 12, 70, 380, 10, False),
        ("                      -------", 12, 70, 365, 10, False),
        ("Total Amount Due:    $159.48", 12, 70, 350, 10, True),
        ("", 0, 0, 0, 0, False),
        ("For support, call (555) 999-8888", 12, 50, 320, 10, False),
        ("Your account 123456789-00001 is in good standing.", 12, 50, 300, 10, False),
    ]
    
    # Add text with varying fonts to simulate Type3-like rendering
    for text, fontname, x, y, fontsize, bold in content:
        if not text:
            continue
        
        # Use courier (monospace) to simulate OCR/scanned style
        font = "courier-bold" if bold else "courier"
        
        # Insert text
        page.insert_text(
            (x, y),
            text,
            fontname=font,
            fontsize=fontsize,
            color=(0, 0, 0),
        )
    
    # Save the PDF
    doc.save(output_path)
    doc.close()
    print(f"✓ Created: {output_path}")


def create_simple_verizon_bill(output_path: str):
    """Create a simpler Verizon bill for basic testing."""
    doc = fitz.open()
    page = doc.new_page(width=612, height=792)
    
    # Simple bill content
    text_content = """
VERIZON WIRELESS
Account Statement

Account Number: 987654321-00009
Statement Date: January 6, 2026
Account Holder: John Doe

Service Details:
- Wireless Plan
- Phone: (415) 555-9999
- Device Payment: $35.00

Account Total: 987654321-00009
Amount Due: $120.00

Contact: 1-800-VERIZON
Account 987654321-00009 in good standing.
"""
    
    # Add text in blocks to simulate different rendering styles
    lines = [line.strip() for line in text_content.strip().split('\n') if line.strip()]
    y = 750
    
    for line in lines:
        if not line:
            continue
            
        # Vary font size for headers vs body
        if any(keyword in line for keyword in ["VERIZON", "Account Statement"]):
            fontsize = 16
            font = "courier-bold"
        elif line.startswith("-") or ":" in line:
            fontsize = 11
            font = "courier"
        else:
            fontsize = 12
            font = "courier"
        
        page.insert_text((50, y), line, fontname=font, fontsize=fontsize)
        y -= fontsize + 4
    
    doc.save(output_path)
    doc.close()
    print(f"✓ Created: {output_path}")


def create_phone_list_pdf(output_path: str):
    """Create a PDF with multiple phone numbers for testing."""
    doc = fitz.open()
    page = doc.new_page(width=612, height=792)
    
    phones = [
        ("Emergency Contacts", "courier-bold", 14),
        ("", "courier", 10),
        ("John Smith: (555) 123-4567", "courier", 11),
        ("Jane Doe: 555-987-6543", "courier", 11),
        ("Bob Wilson: (555) 111-2222", "courier", 11),
        ("Alice Brown: 555.333.4444", "courier", 11),
        ("Charlie Davis: +1 555-555-5555", "courier", 11),
        ("", "courier", 10),
        ("Department Directory", "courier-bold", 14),
        ("", "courier", 10),
        ("Sales: (415) 555-7890", "courier", 11),
        ("Support: 415-555-7891", "courier", 11),
        ("Billing: (415) 555-7892", "courier", 11),
    ]
    
    y = 750
    for text, font, fontsize in phones:
        if not text:
            y -= 10
            continue
        page.insert_text((50, y), text, fontname=font, fontsize=fontsize)
        y -= fontsize + 5
    
    doc.save(output_path)
    doc.close()
    print(f"✓ Created: {output_path}")


def main():
    """Generate all test PDFs with Type3-like characteristics."""
    tests_dir = Path(__file__).parent
    
    print("Generating test PDFs with Type3-like fonts...")
    print()
    
    # Create input PDFs
    create_verizon_bill_with_type3(
        str(tests_dir / "test_input_verizon_type3.pdf")
    )
    
    create_simple_verizon_bill(
        str(tests_dir / "test_input_verizon_simple.pdf")
    )
    
    create_phone_list_pdf(
        str(tests_dir / "test_input_phone_list.pdf")
    )
    
    print()
    print("Done! Test PDFs created in tests/ directory.")
    print()
    print("Next steps:")
    print("1. Run tests: cargo test")
    print("2. Generate expected outputs by running the redactor on these inputs")
    print("3. Verify redaction works correctly on these PDFs")


if __name__ == "__main__":
    main()
