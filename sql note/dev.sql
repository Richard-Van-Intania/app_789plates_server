SELECT price_history.price_history_id,
    price_history.plates_id,
    price_history.price,
    COUNT(lp.liked_plates_id) AS liked_plates_id_count,
    COUNT(sp.saved_plates_id) AS saved_plates_id_count,
    COUNT(lp.liked_plates_id) + COUNT(sp.saved_plates_id) AS reacts_count,
    ROW_NUMBER() OVER (
        PARTITION BY price_history.plates_id
        ORDER BY price_history.price_history_id DESC
    ) AS rownumber
FROM public.price_history
    INNER JOIN public.plates ON plates.plates_id = price_history.plates_id
    AND plates.users_id = 42
    LEFT JOIN public.liked_plates AS lp ON lp.plates_id = price_history.plates_id
    LEFT JOIN public.saved_plates AS sp ON sp.plates_id = price_history.plates_id
GROUP BY price_history.price_history_id,
    price_history.plates_id,
    price_history.price