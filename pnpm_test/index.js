'use strict'
const symlinkDir = require('symlink-dir')
const path = require('path')

symlinkDir('src', 'node_modules/src')
  .then((result) => {
    console.log(result)
    //> { reused: false }

    return symlinkDir('src', 'node_modules/src')
  })
  .then((result) => {
    console.log(result)
    //> { reused: true }
  })
  .catch((err) => console.error(err))
