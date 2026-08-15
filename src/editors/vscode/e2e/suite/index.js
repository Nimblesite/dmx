'use strict';

// The entry point VS Code's extension test host calls [editor.extension.e2e].
// One file of tests, run in order: the suite drives one workspace through its
// whole life — open, generate, edit, break, fix, command — so each test builds
// on the state the previous one proved.

const path = require('node:path');
const Mocha = require('mocha');

function run() {
  const mocha = new Mocha({ ui: 'bdd', color: true, timeout: 120_000 });
  mocha.addFile(path.join(__dirname, 'watch.e2e.js'));
  return new Promise((resolve, reject) => {
    mocha.run((failures) =>
      failures === 0 ? resolve() : reject(new Error(`${failures} VSIX e2e test(s) failed`)),
    );
  });
}

module.exports = { run };
