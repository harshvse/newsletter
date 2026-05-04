-- Seed test data for blog posts and categories
-- This migration will be run for all databases including test databases

INSERT INTO categories (id, category_name, created_at, updated_at) VALUES
    (1, 'Databricks', NOW(), NOW()),
    (2, 'Rust', NOW(), NOW()),
    (3, 'C++', NOW(), NOW()),
    (4, 'GoLang', NOW(), NOW()),
    (5, 'Python', NOW(), NOW()),
    (6, 'CI/CD', NOW(), NOW())
ON CONFLICT (id) DO NOTHING;

-- Reset category sequence to ensure new inserts work correctly
SELECT setval('categories_id_seq', (SELECT MAX(id) FROM categories) + 1);

INSERT INTO blog_posts (id, title, summary, content, author, status, published_at, created_at, updated_at) VALUES
    (1, 'Getting Started with Rust', 'Learn the basics of Rust programming language', '# Rust Basics\n\nRust is a systems programming language that runs blazingly fast and prevents segfaults.', 'Harsh Verma', 'published', NOW(), NOW(), NOW()),
    (2, 'Advanced Databricks Optimization', 'Tips and tricks for optimizing Databricks clusters', '# Databricks Optimization\n\nLearn how to optimize your Databricks clusters for better performance.', 'Harsh Verma', 'published', NOW(), NOW(), NOW()),
    (3, 'Python Best Practices', 'Writing clean and maintainable Python code', '# Python Best Practices\n\nFollow these practices for writing better Python code.', 'Harsh Verma', 'draft', NULL, NOW(), NOW()),
    (4, 'CI/CD Pipeline Setup', 'Setting up continuous integration and deployment', '# CI/CD Pipelines\n\nLearn how to set up effective CI/CD pipelines for your projects.', 'Harsh Verma', 'published', NOW(), NOW(), NOW())
ON CONFLICT (id) DO NOTHING;

-- Reset post sequence
SELECT setval('blog_posts_id_seq', (SELECT MAX(id) FROM blog_posts) + 1);

INSERT INTO post_categories (post_id, category_id) VALUES
    (1, 2),  -- Rust post -> Rust category
    (2, 1),  -- Databricks post -> Databricks category
    (3, 5),  -- Python post -> Python category
    (4, 6)   -- CI/CD post -> CI/CD category
ON CONFLICT DO NOTHING;
