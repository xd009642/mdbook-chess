(() => {
  for (const root of document.querySelectorAll('.chess-replay:not([data-ready])')) {
    root.dataset.ready = 'true';
    const frames = [...root.querySelectorAll('.chess-frame')];
    const controls = root.querySelector('.chess-controls');
    const slider = controls.querySelector('input');
    const status = controls.querySelector('output');
    const previous = controls.querySelector('[data-action="previous"]');
    const next = controls.querySelector('[data-action="next"]');
    const play = controls.querySelector('[data-action="play"]');
    let index = 0;
    let timer = null;
    slider.max = frames.length - 1;
    controls.hidden = false;
    const show = () => {
      frames.forEach((frame, i) => { frame.hidden = i !== index; });
      slider.value = index;
      status.textContent = frames[index].dataset.label;
      previous.disabled = index === 0;
      next.disabled = index === frames.length - 1;
    };
    const stop = () => {
      clearInterval(timer);
      timer = null;
      play.textContent = 'Play';
      play.setAttribute('aria-label', 'Play moves');
    };
    previous.addEventListener('click', () => { stop(); index--; show(); });
    next.addEventListener('click', () => { stop(); index++; show(); });
    slider.addEventListener('input', () => { stop(); index = Number(slider.value); show(); });
    play.addEventListener('click', () => {
      if (timer !== null) { stop(); return; }
      if (index === frames.length - 1) { index = 0; show(); }
      play.textContent = 'Pause';
      play.setAttribute('aria-label', 'Pause moves');
      timer = setInterval(() => {
        if (index < frames.length - 1) index++;
        else if (root.dataset.repeat === 'true') index = 0;
        else { stop(); return; }
        show();
      }, Number(root.dataset.delay));
    });
    document.addEventListener('visibilitychange', () => { if (document.hidden) stop(); });
    window.addEventListener('beforeprint', stop);
    show();
  }
  for (const root of document.querySelectorAll('.chess-gif:not([data-ready])')) {
    root.dataset.ready = 'true';
    const controls = root.querySelector('.chess-controls');
    const button = controls.querySelector('button');
    const poster = root.querySelector('.chess-poster');
    const animation = root.querySelector('.chess-animation');
    controls.hidden = false;
    const stop = () => {
      animation.hidden = true;
      animation.removeAttribute('src');
      poster.hidden = false;
      button.textContent = 'Play GIF';
    };
    button.addEventListener('click', () => {
      if (!animation.hidden) { stop(); return; }
      animation.src = animation.dataset.src;
      animation.hidden = false;
      poster.hidden = true;
      button.textContent = 'Stop GIF';
    });
    document.addEventListener('visibilitychange', () => { if (document.hidden) stop(); });
    window.addEventListener('beforeprint', stop);
  }
})();
