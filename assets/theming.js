(() => {
  const THEME_LOCAL_STORAGE_KEY = 'theme';
  const DEFAULT_THEME = 'dark';

  let theme = DEFAULT_THEME;

  let persistTheme = () => {
    window.localStorage.setItem(THEME_LOCAL_STORAGE_KEY, theme);
  };

  let updateBodyTheme = () => {
    Array.from(document.body.classList).forEach((className) => {
      if (className.startsWith('theme-')) {
        document.body.classList.remove(className);
      }
    });
    if (theme != DEFAULT_THEME) {
      document.body.classList.add('theme-' + theme);
    }
  };

  window.changeColorTheme = (newTheme) => {
    if (newTheme != theme) {
      theme = newTheme;
      persistTheme();
      updateBodyTheme();
    }
  };

  window.addEventListener("DOMContentLoaded", (_ev) => {
    theme = window.localStorage.getItem(THEME_LOCAL_STORAGE_KEY);
    if (theme == null) {
      theme = DEFAULT_THEME;
    }
    persistTheme();
    updateBodyTheme();
  });
})();
