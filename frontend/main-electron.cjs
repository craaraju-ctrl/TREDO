const { app, BrowserWindow, Menu } = require('electron');
const path = require('path');

let mainWindow;

function createWindow() {
  mainWindow = new BrowserWindow({
    width: 1280,
    height: 800,
    minWidth: 1024,
    minHeight: 768,
    title: "TREDO Cockpit — Sethu Orchestration",
    webPreferences: {
      nodeIntegration: false,
      contextIsolation: true,
      preload: path.join(__dirname, 'preload.js') // optional placeholder for security
    },
    backgroundColor: '#0a0d16',
    show: false
  });

  // Check if we are in development mode
  const isDev = process.env.NODE_ENV === 'development' || !app.isPackaged;

  if (isDev) {
    // Load local Vite dev server
    mainWindow.loadURL('http://localhost:3000').catch((err) => {
      console.warn("Failed to load localhost:3000, loading fallback static build.", err);
      mainWindow.loadFile(path.join(__dirname, 'dist', 'index.html'));
    });
    // Open Chrome DevTools in development
    mainWindow.webContents.openDevTools();
  } else {
    // Load static packaged dist build
    mainWindow.loadFile(path.join(__dirname, 'dist', 'index.html')).catch((err) => {
      console.error("Failed to load packaged index.html:", err);
    });
  }

  // Optimize window loading appearance
  mainWindow.once('ready-to-show', () => {
    mainWindow.show();
  });

  mainWindow.on('closed', () => {
    mainWindow = null;
  });

  // Create clean application menu
  createApplicationMenu(isDev);
}

function createApplicationMenu(isDev) {
  const template = [
    {
      label: 'App',
      submenu: [
        { role: 'about' },
        { type: 'separator' },
        { role: 'quit' }
      ]
    },
    {
      label: 'Edit',
      submenu: [
        { role: 'undo' },
        { role: 'redo' },
        { type: 'separator' },
        { role: 'cut' },
        { role: 'copy' },
        { role: 'paste' },
        { role: 'selectAll' }
      ]
    },
    {
      label: 'View',
      submenu: [
        { role: 'reload' },
        { role: 'forceReload' },
        { type: 'separator' },
        { role: 'togglefullscreen' },
        ...(isDev ? [{ role: 'toggleDevTools' }] : [])
      ]
    },
    {
      label: 'Window',
      submenu: [
        { role: 'minimize' },
        { role: 'zoom' },
        { type: 'separator' },
        { role: 'close' }
      ]
    }
  ];

  const menu = Menu.buildFromTemplate(template);
  Menu.setApplicationMenu(menu);
}

app.whenReady().then(() => {
  createWindow();

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow();
    }
  });
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});
