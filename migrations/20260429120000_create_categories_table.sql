-- Create categories table
CREATE TABLE categories (
    id SERIAL PRIMARY KEY,
    category_name TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
