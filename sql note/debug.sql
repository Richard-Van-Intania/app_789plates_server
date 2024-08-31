SELECT price_history.price_history_id,
    price_history.plates_id,
    price_history.price,
    COUNT(lp.liked_plates_id) + COUNT(sp.saved_plates_id) AS react_count,
    ROW_NUMBER() OVER (
        PARTITION BY price_history.plates_id
        ORDER BY price_history.price_history_id DESC
    ) AS row_num
FROM public.price_history
    LEFT JOIN public.liked_plates AS lp ON lp.plates_id = price_history.plates_id
    LEFT JOIN public.saved_plates AS sp ON sp.plates_id = price_history.plates_id
GROUP BY price_history.price_history_id,
    price_history.plates_id,
    price_history.price