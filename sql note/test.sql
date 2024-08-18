INSERT INTO
    public.verification (reference, code, expire)
VALUES
    (1, 2, 3) RETURNING verification_id,
    reference