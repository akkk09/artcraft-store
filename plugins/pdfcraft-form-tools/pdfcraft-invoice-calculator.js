/**
 * Form Calculator & Validator for PdfCraft (ISO 32000 / Acrobat JavaScript)
 * Compatible with PdfCraft form engine (Boa sandbox).
 */

// --- 1. Line Item & Subtotal Calculator ---
function calculateInvoiceTotal() {
    var subtotal = 0;
    for (var i = 1; i <= 10; i++) {
        var qtyField = this.getField("Qty." + i);
        var priceField = this.getField("Price." + i);
        var lineField = this.getField("Total." + i);
        if (qtyField && priceField && lineField) {
            var qty = Number(qtyField.value) || 0;
            var price = Number(priceField.value) || 0;
            var lineTotal = qty * price;
            lineField.value = lineTotal;
            subtotal += lineTotal;
        }
    }
    var subtotalField = this.getField("Subtotal");
    if (subtotalField) subtotalField.value = subtotal;

    var taxRateField = this.getField("TaxRate");
    var taxRate = (taxRateField ? Number(taxRateField.value) : 0) / 100;
    var tax = subtotal * taxRate;
    var taxField = this.getField("Tax");
    if (taxField) taxField.value = tax;

    var grandTotalField = this.getField("GrandTotal");
    if (grandTotalField) grandTotalField.value = subtotal + tax;
}

// --- 2. Format Currency Handler ---
function formatCurrency(fieldValue) {
    var num = Number(fieldValue) || 0;
    if (typeof util !== "undefined" && util.printf) {
        return util.printf("$%.2f", num);
    }
    return "$" + num.toFixed(2);
}

// --- 3. Keystroke Validator for Number Fields ---
function validateNumericKeystroke(event) {
    if (event.willCommit) {
        var n = Number(event.value);
        if (isNaN(n) || n < 0) {
            if (typeof app !== "undefined" && app.alert) {
                app.alert("Please enter a valid positive number.");
            }
            event.rc = false;
        }
    }
}

// --- 4. Email Validator ---
function validateEmail(event) {
    if (event.willCommit && event.value) {
        var re = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
        if (!re.test(event.value)) {
            if (typeof app !== "undefined" && app.alert) {
                app.alert("Please enter a valid email address.");
            }
            event.rc = false;
        }
    }
}
