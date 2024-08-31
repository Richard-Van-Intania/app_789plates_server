WITH latest_price AS (
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
        LEFT JOIN public.liked_plates AS lp ON lp.plates_id = price_history.plates_id
        LEFT JOIN public.saved_plates AS sp ON sp.plates_id = price_history.plates_id
    GROUP BY price_history.price_history_id,
        price_history.plates_id,
        price_history.price
)
SELECT plates.plates_id,
    plates.front_text,
    plates.plates_type_id,
    plates.plates_uri,
    plates.total,
    plates.front_number,
    plates.back_number,
    plates.users_id,
    plates.special_front_id,
    plates.province_id,
    plates.information,
    latest_price.price,
    users.name,
    users.profile_uri,
    liked_plates.liked_plates_id,
    saved_plates.saved_plates_id,
    liked_store.liked_store_id,
    saved_store.saved_store_id,
    latest_price.liked_plates_id_count,
    latest_price.saved_plates_id_count,
    latest_price.reacts_count,
    latest_price.rownumber
FROM latest_price
    INNER JOIN public.plates ON plates.plates_id = latest_price.plates_id
    INNER JOIN public.users ON users.users_id = plates.users_id
    LEFT JOIN public.liked_plates ON liked_plates.plates_id = plates.plates_id
    AND liked_plates.users_id = 10
    LEFT JOIN public.saved_plates ON saved_plates.plates_id = plates.plates_id
    AND saved_plates.users_id = 10
    LEFT JOIN public.liked_store ON liked_store.store_id = plates.users_id
    AND liked_store.users_id = 10
    LEFT JOIN public.saved_store ON saved_store.store_id = plates.users_id
    AND saved_store.users_id = 10
WHERE latest_price.rownumber = 1
    AND is_selling IS TRUE
    AND is_temporary IS NOT TRUE
    AND latest_price.price <= 1000000
    AND plates.province_id IN (1, 2)
    AND plates.plates_type_id IN (1, 2, 5, 6)
ORDER BY latest_price.reacts_count DESC
LIMIT 500 OFFSET 0