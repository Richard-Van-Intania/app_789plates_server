SELECT *
FROM public.plates
WHERE plates_type_id IN (
        SELECT unnest (1)
    )
    AND province_id IN (
        SELECT unnest (2)
    )
ORDER BY plates_id ASC