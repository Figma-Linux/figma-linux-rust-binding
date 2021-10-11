const { getFonts: _getFonts } = require("./index.node");

const getFonts = async dirs => new Promise(resolve => {
  _getFonts(dirs, fonts => {
    resolve(fonts);
  });
});

export {
  getFonts,
}
