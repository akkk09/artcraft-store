# Form Calculator & Validator for PdfCraft

Acrobat JavaScript (ISO 32000) scripts for interactive PDF forms in PdfCraft.

## Features
- **Dynamic Field Calculation**: Computes invoice line items, tax, discounts, and grand totals automatically.
- **Currency & Number Formatting**: Formats inputs to localized decimal currency.
- **Form Field Validation**: Validates email addresses, phone numbers, and numeric inputs on commit.
- **Sandboxed Execution**: Compatible with PdfCraft's pure-Rust Boa JavaScript runtime.

## Installation
In PdfCraft:
1. Open **Tools ▸ Form Edit**.
2. Select target field ▸ **Properties ▸ Actions / Calculate**.
3. Paste the function calls into the field script editor, or attach `pdfcraft-invoice-calculator.js` as a Document-Level JavaScript.
