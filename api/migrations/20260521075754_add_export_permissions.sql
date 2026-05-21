-- Up migration
INSERT INTO permissions (id, name, description, model_group) VALUES 
    (gen_random_uuid(), 'can_export_sale_pdf', 'Permet d''exporter l''historique des ventes en PDF', 'sales'),
    (gen_random_uuid(), 'can_export_sale_excel', 'Permet d''exporter l''historique des ventes en Excel/CSV', 'sales'),
    (gen_random_uuid(), 'can_print_sale_receipt', 'Permet d''imprimer un reçu de vente', 'sales')
ON CONFLICT (name) DO NOTHING;
