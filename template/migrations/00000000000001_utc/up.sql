-- Set timezone to UTC
DO $$
BEGIN
   EXECUTE 'ALTER DATABASE "'||current_database()||'" SET timezone TO ''UTC''';
END
$$;
SELECT pg_reload_conf();