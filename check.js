const { getFonts } = require("./index.js");

(async () => {
  const fonts = await getFonts(["/home/ruut/.local/share/fonts/Segoe"]);

  console.log("fonts: ", fonts);
})();
