-- Actuator Mesh Database Schema
-- Implements the Deterministic URE-UUID (DU-UUID) content-addressable registry
-- and the Quantum-State Actuator machine.

CREATE TABLE IF NOT EXISTS actuators (
    resource_id UUID PRIMARY KEY,         -- The DU-UUID (Content Hash)
    parent_id UUID REFERENCES actuators(resource_id), -- Lineage for mutation/evolution
    complexity_score DECIMAL(3,2) NOT NULL DEFAULT 0.5, -- 0.0 (Surface) to 1.0 (Deep)
    
    -- Actuator Metadata
    resource_type VARCHAR(50) NOT NULL,  -- identity, skill, config, job, hardware, tool, doc
    category VARCHAR(100),               -- L (Local), E (External), C (Cloud)
    location_flag VARCHAR(10),           -- L, E, C
    
    -- Performance Metrics (from .ure manifest)
    quality_of_product DECIMAL(3,2) DEFAULT 0.5,
    success_rate DECIMAL(3,2) DEFAULT 1.0,
    avg_execution_time_ms INTEGER DEFAULT 5000,
    tokens_per_task INTEGER DEFAULT 1000,
    
    -- State Management
    current_state VARCHAR(50) DEFAULT 'quantic', -- quantic, surface, deep, floating, externalized
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS actuator_correlations (
    id SERIAL PRIMARY KEY,
    source_id UUID REFERENCES actuators(resource_id),
    target_id UUID REFERENCES actuators(resource_id),
    relationship_type VARCHAR(50),       -- requires, provides, complements, derived_from
    strength DECIMAL(3,2) NOT NULL DEFAULT 1.0,
    usage_count INTEGER DEFAULT 0,
    last_used TIMESTAMP DEFAULT NOW(),
    expiry_date TIMESTAMP,
    
    CONSTRAINT unique_correlation UNIQUE (source_id, target_id, relationship_type)
);

CREATE TABLE IF NOT EXISTS actuator_history (
    id SERIAL PRIMARY KEY,
    resource_id UUID REFERENCES actuators(resource_id),
    old_resource_id UUID,                -- For mutation tracking
    change_reason TEXT,                  -- 'initial_creation', 'mutation', 'synthesis'
    strength_delta DECIMAL(3,2),
    timestamp TIMESTAMP DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS behavioral_vectors (
    id SERIAL PRIMARY KEY,
    vector_name VARCHAR(50) UNIQUE,      -- 'quickest', 'smartest', 'direct'
    priority_weight DECIMAL(3,2),
    token_budget_limit INTEGER,
    description TEXT
);

-- Indices for Mesh Performance
CREATE INDEX idx_actuators_complexity ON actuators(complexity_score);
CREATE INDEX idx_actuators_type ON actuators(resource_type);
CREATE INDEX idx_correlations_source ON actuator_correlations(source_id);
CREATE INDEX idx_correlations_target ON actuator_correlations(target_id);
CREATE INDEX idx_correlations_strength ON actuator_correlations(strength DESC);
