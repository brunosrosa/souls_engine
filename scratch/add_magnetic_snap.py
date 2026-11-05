with open('crates/souls_server/src/dashboard_ui.html', 'r', encoding='utf-8') as f:
    content = f.read()

target = '''    // NAVEGAÇÃO ENTRE ABAS
    function switchTab(tabId) {'''

code = '''    // SLIDERS COM FEEDBACK TÁTIL & SNAP MAGNÉTICO AO DEFAULT
    function initSlidersMagneticSnap() {
      document.querySelectorAll('input[type="range"][data-default]').forEach(slider => {
        const defVal = parseFloat(slider.getAttribute('data-default'));
        const snapDelta = parseFloat(slider.getAttribute('data-snap-delta')) || ((parseFloat(slider.max) - parseFloat(slider.min)) * 0.04);
        const tick = document.querySelector(`.slider-default-tick[data-slider="${slider.id}"]`);
        
        const updateTickStyle = (val) => {
          if (tick) {
            if (Math.abs(val - defVal) <= 0.001) {
              tick.classList.add('at-default');
            } else {
              tick.classList.remove('at-default');
            }
          }
        };

        slider.addEventListener('input', () => {
          let cur = parseFloat(slider.value);
          if (Math.abs(cur - defVal) <= snapDelta) {
            slider.value = defVal;
            cur = defVal;
          }
          updateTickStyle(cur);
        });

        if (tick) {
          tick.addEventListener('click', () => {
            slider.value = defVal;
            slider.dispatchEvent(new Event('input', { bubbles: true }));
            updateTickStyle(defVal);
          });
        }
        updateTickStyle(parseFloat(slider.value));
      });
    }

    // NAVEGAÇÃO ENTRE ABAS
    function switchTab(tabId) {'''

if target in content:
    content = content.replace(target, code, 1)
    content = content.replace('\r\n', '\n')
    with open('crates/souls_server/src/dashboard_ui.html', 'w', encoding='utf-8', newline='\n') as f:
        f.write(content)
    print("Added initSlidersMagneticSnap successfully!")
else:
    print("Warning: target not found!")

