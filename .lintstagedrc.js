const path = require('path');

const buildLintCommand = (filenames) =>
  `yarn run lint ${filenames
    .map((f) => path.relative(process.cwd(), f))
    .join(' ')}`;

const buildFormatCommand = (filenames) =>
  `prettier ${filenames
    .map((f) => path.relative(process.cwd(), f))
    .join(' ')} `;

module.exports = {
  '**/*.{js,jsx,ts,tsx}': [buildLintCommand],
  '**/*.{js,jsx,ts,tsx,json,md}': [buildFormatCommand],
};
