-- This file should undo anything in `up.sql`
DO $$
BEGIN
   EXECUTE 'ALTER DATABASE "'||current_database()||'" SET timezone TO ''UTC''';
END
$$;
SELECT pg_reload_conf();