-- seeds/fake_bank_accounts.sql

INSERT INTO fake_bank_accounts (user_id, account_number, ifsc, name)
VALUES
    ('550e8400-e29b-41d4-a716-446655440001', '1234567890', 'HDFC0001234', 'John Doe'),
    ('550e8400-e29b-41d4-a716-446655440002', '2345678901', 'ICIC0002345', 'Jane Smith'),
    ('550e8400-e29b-41d4-a716-446655440003', '3456789012', 'SBIN0003456', 'Alice Johnson'),
    ('550e8400-e29b-41d4-a716-446655440004', '4567890123', 'KKBK0004567', 'Bob Brown'),
    ('550e8400-e29b-41d4-a716-446655440005', '5678901234', 'PUNB0005678', 'Charlie Davis'),
    ('550e8400-e29b-41d4-a716-446655440006', '6789012345', 'CNRB0006789', 'Diana Wilson'),
    ('550e8400-e29b-41d4-a716-446655440007', '7890123456', 'SYNB0007890', 'Eve Martinez'),
    ('550e8400-e29b-41d4-a716-446655440008', '8901234567', 'UTIB0008901', 'Frank Taylor'),
    ('550e8400-e29b-41d4-a716-446655440009', '9012345678', 'UCBA0009012', 'Grace Lee'),
    ('550e8400-e29b-41d4-a716-44665544000a', 'LOW_BALANCE', 'YESB0000001', 'Henry Walker')
ON CONFLICT (user_id) DO UPDATE
SET
    account_number = EXCLUDED.account_number,
    ifsc = EXCLUDED.ifsc,
    name = EXCLUDED.name;

-- Optional: Verify
SELECT user_id, name, account_number, ifsc FROM fake_bank_accounts;



-- Optional: Seed wallet balances too
INSERT INTO wallets (user_id, balance, version)
VALUES
    ('550e8400-e29b-41d4-a716-446655440001', 100000, 0),
    ('550e8400-e29b-41d4-a716-446655440002', 250000, 0),
    -- ... add for all 10
ON CONFLICT (user_id) DO UPDATE
SET balance = EXCLUDED.balance, version = 0;