#!/usr/bin/env node

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const checkMode = process.argv.includes('--check');

console.log(checkMode ? 'Checking SQLx prepared queries...' : 'Preparing database for SQLx...');

// Change to backend directory
const backendDir = path.join(__dirname, '..', 'crates/db');
process.chdir(backendDir);

// Use PostgreSQL database (running in container)
const databaseUrl = 'postgres://vibekanban:vibekanban123@localhost:5632/vibe_kanban';

console.log(`Using database: ${databaseUrl}`);

try {
  // Run migrations
  console.log('Running migrations...');
  execSync('cargo sqlx migrate run', {
    stdio: 'inherit',
    env: { ...process.env, DATABASE_URL: databaseUrl }
  });

  // Prepare queries
  const sqlxCommand = checkMode ? 'cargo sqlx prepare --check' : 'cargo sqlx prepare';
  console.log(checkMode ? 'Checking prepared queries...' : 'Preparing queries...');
  execSync(sqlxCommand, {
    stdio: 'inherit',
    env: { ...process.env, DATABASE_URL: databaseUrl }
  });

  console.log(checkMode ? 'SQLx check complete!' : 'Database preparation complete!');

} catch (error) {
  console.error('Database preparation failed:', error.message);
  process.exit(1);
}