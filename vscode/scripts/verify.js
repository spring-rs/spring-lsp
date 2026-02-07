#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

console.log('✅ Verifying extension configuration...');

// 检查必需的文件
const requiredFiles = [
  'package.json',
  'dist/extension.js',
  'resources/logo.png'
];

let allFilesExist = true;

for (const file of requiredFiles) {
  const filePath = path.join(__dirname, '..', file);
  if (!fs.existsSync(filePath)) {
    console.error(`❌ Missing required file: ${file}`);
    allFilesExist = false;
  } else {
    console.log(`✓ Found: ${file}`);
  }
}

if (!allFilesExist) {
  console.error('\n❌ Verification failed!');
  process.exit(1);
}

console.log('\n✅ All checks passed!');
