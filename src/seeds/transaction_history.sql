-- seeds/transaction_history.sql
-- Seed 50 fake transactions for 5 users

-- Ensure users exist (adjust UUIDs to match your seeded users)
DO $$
DECLARE
    user_ids UUID[] := ARRAY[
        '550e8400-e29b-41d4-a716-446655440001',
        '550e8400-e29b-41d4-a716-446655440002',
        '550e8400-e29b-41d4-a716-446655440003',
        '550e8400-e29b-41d4-a716-446655440004',
        '550e8400-e29b-41d4-a716-446655440005'
    ];
    i INT;
    tx_count INT := 50;
    from_idx INT;
    to_idx INT;
    amount INT;
    tx_time TIMESTAMP;
BEGIN
    FOR i IN 1..tx_count LOOP
        -- Random from/to user (not self)
        from_idx := floor(random() * 5) + 1;
        to_idx := floor(random() * 5) + 1;
        WHILE to_idx = from_idx LOOP
            to_idx := floor(random() * 5) + 1;
        END LOOP;

        -- Random amount: ₹10 to ₹5,000
        amount := floor(random() * 499000 + 1000); -- in paise

        -- Random timestamp: last 30 days
        tx_time := NOW() - (random() * INTERVAL '30 days');

        -- Insert transaction
        INSERT INTO transaction_journal (
            tx_id,
            from_user_id,
            to_user_id,
            amount,
            status,
            idempotency_key,
            created_at
        ) VALUES (
            gen_random_uuid(),
            user_ids[from_idx],
            user_ids[to_idx],
            amount,
            'SUCCESS',
            'idemp_' || gen_random_uuid(),
            tx_time
        )
        ON CONFLICT (idempotency_key) DO NOTHING;
    END LOOP;

    RAISE NOTICE '✅ Inserted 50 fake transactions';
END $$;

-- Optional: Verify
SELECT
    tj.tx_id,
    tj.from_user_id,
    tj.to_user_id,
    tj.amount,
    tj.created_at,
    u1.mobile_hash as from_mobile,
    u2.mobile_hash as to_mobile
FROM transaction_journal tj
JOIN users u1 ON tj.from_user_id = u1.id
JOIN users u2 ON tj.to_user_id = u2.id
ORDER BY tj.created_at DESC
LIMIT 10;


-- Flag 3 random transactions as high-risk
INSERT INTO fraud_flags (tx_id, risk_score, reason)
SELECT tx_id, 85, 'High amount + new device'
FROM transaction_journal
ORDER BY RANDOM()
LIMIT 3
ON CONFLICT (tx_id) DO NOTHING;