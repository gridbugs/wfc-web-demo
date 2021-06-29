import('../pkg').catch(console.error).then(wasm => {
  console.log(wasm);
  const c = document.querySelector('#c');
  const ctx = c.getContext('2d');
  const seed = parseInt(Math.pow(2, 32) * Math.random())

  const params = new URLSearchParams(window.location.search);
  let image = params.get("image");
  if (image === null) {
    image = "flowers";
  }
  let rotate = parseInt(params.get("rotate"));
  if (isNaN(rotate)) {
    rotate = 1;
  }
  rotate = rotate != 0;

  let roughNumCells = parseInt(params.get("cells"));
  if (isNaN(roughNumCells)) {
    roughNumCells = 4000;
  }
  const cellSide = Math.sqrt((window.innerWidth * window.innerHeight) / roughNumCells)

  const widthInCells = parseInt(window.innerWidth / cellSide)
  const heightInCells = parseInt(window.innerHeight / cellSide)

  const wfc = new wasm.Wfc(widthInCells, heightInCells, seed, image, rotate);

  let timeout = null;
  const finishedDelayMs = 5000;

  function go() {
    if (wfc.tick(ctx, c.width, c.height)) {
      timeout = setTimeout(() => {
        wfc.reset();
        go();
      }, finishedDelayMs);
    } else {
      requestAnimationFrame(go);
    }
  }
  go();

  document.onkeydown = (_e) => {
    if (finishedDelayMs !== null) {
      clearTimeout(timeout);
    }
    wfc.reset();
  }
})

