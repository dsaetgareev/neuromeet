CREATE TABLE conferences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conference UUID NOT NULL,
    user_id TEXT NOT NULL,
    start_time TIMESTAMP NOT NULL,
    end_time TIMESTAMP,
    status TEXT NOT NULL CHECK (status IN ('active', 'completed')),
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- Индекс для быстрого поиска конференций по пользователю
CREATE INDEX idx_conferences_conference ON conferences(conference);

-- Триггер для автоматического обновления поля updated_at
CREATE FUNCTION update_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_timestamp
BEFORE UPDATE ON conferences
FOR EACH ROW
EXECUTE FUNCTION update_timestamp();
