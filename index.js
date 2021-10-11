const { getFonts } = require("./index.node");

let dirs = [
  `${process.env.HOME}/.local/share/bad_fonts`,
  `${process.env.HOME}/.local/share/fonts`,
  '/usr/share/fonts'
];

console.log('getFonts: ', getFonts);

getFonts(dirs, result => {
  console.log('result: ', result);
});

console.log('last log');
