-- Rulecraft Initial Schema
-- D&D 2024 Rules Database

-- Rules table
CREATE TABLE IF NOT EXISTS rules (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    category TEXT NOT NULL,
    subcategory TEXT,
    content TEXT NOT NULL,
    source TEXT NOT NULL,
    page INTEGER,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Full-text search index
CREATE VIRTUAL TABLE IF NOT EXISTS rules_fts USING fts5(
    title,
    content,
    category,
    content='rules',
    content_rowid='rowid'
);

-- Triggers to keep FTS index in sync
CREATE TRIGGER IF NOT EXISTS rules_ai AFTER INSERT ON rules BEGIN
    INSERT INTO rules_fts(rowid, title, content, category)
    VALUES (NEW.rowid, NEW.title, NEW.content, NEW.category);
END;

CREATE TRIGGER IF NOT EXISTS rules_ad AFTER DELETE ON rules BEGIN
    INSERT INTO rules_fts(rules_fts, rowid, title, content, category)
    VALUES ('delete', OLD.rowid, OLD.title, OLD.content, OLD.category);
END;

CREATE TRIGGER IF NOT EXISTS rules_au AFTER UPDATE ON rules BEGIN
    INSERT INTO rules_fts(rules_fts, rowid, title, content, category)
    VALUES ('delete', OLD.rowid, OLD.title, OLD.content, OLD.category);
    INSERT INTO rules_fts(rowid, title, content, category)
    VALUES (NEW.rowid, NEW.title, NEW.content, NEW.category);
END;

-- Indexes
CREATE INDEX IF NOT EXISTS idx_rules_category ON rules(category);
CREATE INDEX IF NOT EXISTS idx_rules_source ON rules(source);

-- Sample data (D&D 2024 basic rules)
INSERT OR IGNORE INTO rules (id, title, category, subcategory, content, source, page, created_at, updated_at)
VALUES
    ('adv-disadv', 'Advantage and Disadvantage', 'Combat', 'Rolling',
     'Sometimes a special ability or spell tells you that you have advantage or disadvantage on an ability check, a saving throw, or an attack roll. When that happens, you roll a second d20 when you make the roll. Use the higher of the two rolls if you have advantage, and use the lower roll if you have disadvantage.',
     'Player''s Handbook 2024', 25, '2024-01-01', '2024-01-01'),

    ('sneak-attack', 'Sneak Attack', 'Combat', 'Class Features',
     'Once per turn, you can deal extra 1d6 damage to one creature you hit with an attack if you have advantage on the attack roll. The attack must use a finesse or a ranged weapon. You don''t need advantage on the attack roll if another enemy of the target is within 5 feet of it, that enemy isn''t incapacitated, and you don''t have disadvantage on the attack roll.',
     'Player''s Handbook 2024', 98, '2024-01-01', '2024-01-01'),

    ('opportunity-attack', 'Opportunity Attack', 'Combat', 'Actions',
     'You can make an opportunity attack when a hostile creature that you can see moves out of your reach. To make the opportunity attack, you use your reaction to make one melee attack against the provoking creature. The attack occurs right before the creature leaves your reach.',
     'Player''s Handbook 2024', 195, '2024-01-01', '2024-01-01'),

    ('concentration', 'Concentration', 'Spellcasting', 'Mechanics',
     'Some spells require you to maintain concentration to keep their magic active. If you lose concentration, such a spell ends. Taking damage can break your concentration. When you take damage while concentrating on a spell, make a Constitution saving throw. The DC equals 10 or half the damage taken, whichever is higher.',
     'Player''s Handbook 2024', 233, '2024-01-01', '2024-01-01'),

    ('cover', 'Cover', 'Combat', 'Environment',
     'Walls, trees, creatures, and other obstacles can provide cover during combat. Half cover grants +2 to AC and Dexterity saving throws. Three-quarters cover grants +5 to AC and Dexterity saving throws. Total cover means a target cannot be targeted directly by an attack or spell.',
     'Player''s Handbook 2024', 196, '2024-01-01', '2024-01-01');
