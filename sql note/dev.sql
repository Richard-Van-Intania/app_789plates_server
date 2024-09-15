WITH latest_price AS (
    SELECT price_history.price_history_id,
        price_history.plates_id,
        price_history.price,
        ROW_NUMBER() OVER (
            PARTITION BY price_history.plates_id
            ORDER BY price_history.price_history_id DESC
        ) AS rownumber,
        plates.users_id
    FROM public.price_history
        INNER JOIN public.plates ON plates.plates_id = price_history.plates_id
        INNER JOIN public.users ON users.users_id = plates.users_id
        AND users.name LIKE '%li%'
)
SELECT users.users_id,
    users.name,
    users.profile_uri,
    users.cover_uri,
    users.information,
    liked_store.liked_store_id,
    saved_store.saved_store_id,
    COUNT(ls.liked_store_id) AS liked_store_id_count,
    COUNT(ss.saved_store_id) AS saved_store_id_count,
    SUM(latest_price.price) AS total_assets,
    COUNT(latest_price.plates_id) AS plates_count,
    AVG(rating.score) AS average_score
FROM latest_price
    INNER JOIN public.users ON users.users_id = latest_price.users_id
    LEFT JOIN public.liked_store ON liked_store.store_id = latest_price.users_id
    AND liked_store.users_id = 10
    LEFT JOIN public.saved_store ON saved_store.store_id = latest_price.users_id
    AND saved_store.users_id = 10
    LEFT JOIN public.rating ON rating.store_id = latest_price.users_id
    LEFT JOIN public.liked_store AS ls ON ls.store_id = latest_price.users_id
    LEFT JOIN public.saved_store AS ss ON ss.store_id = latest_price.users_id
WHERE latest_price.rownumber = 1
GROUP BY users.users_id,
    liked_store.liked_store_id,
    saved_store.saved_store_id