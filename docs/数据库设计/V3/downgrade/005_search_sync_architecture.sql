DROP TRIGGER IF EXISTS trg_partner_search_document_updated_at ON search.partner_search_document;
DROP TRIGGER IF EXISTS trg_mutual_recognition_search_refresh ON ecosystem.mutual_recognition;
DROP TRIGGER IF EXISTS trg_partner_search_refresh ON ecosystem.partner;

DROP FUNCTION IF EXISTS search.tg_refresh_partner_search_document();
DROP FUNCTION IF EXISTS search.refresh_partner_search_document_by_id(uuid);

DROP TABLE IF EXISTS search.partner_search_document CASCADE;
