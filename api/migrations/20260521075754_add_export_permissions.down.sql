-- Down migration
DELETE FROM permissions WHERE name IN ('can_export_sale_pdf', 'can_export_sale_excel', 'can_print_sale_receipt');
