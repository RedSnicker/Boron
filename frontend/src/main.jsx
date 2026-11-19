import React, {useState} from 'react';
import ReactDOM from 'react-dom/client';
import { BrowserRouter } from 'react-router-dom';
import App from './App';
import { Theme, defineTheme } from '@astryxdesign/core/theme';
import { neutralTheme } from '@astryxdesign/theme-neutral/built';
import '@astryxdesign/theme-neutral/theme.css';

// Astryx setup - required for components to render styled
import '@astryxdesign/core/reset.css';
import '@astryxdesign/core/astryx.css';

const boronTheme = defineTheme({name:'boron', extends:neutralTheme, motion:{tempo:0.75,character:'crisp',fast:100,medium:140,slow:180,ratio:0.75}});

function ThemedApp(){const [mode,setMode]=useState(localStorage.getItem('boron_theme')||'dark');const change=value=>{localStorage.setItem('boron_theme',value);setMode(value);};return <Theme theme={boronTheme} mode={mode}><App theme={mode} setTheme={change}/></Theme>;}

ReactDOM.createRoot(document.getElementById('root')).render(
  <React.StrictMode>
    <BrowserRouter>
      <ThemedApp />
    </BrowserRouter>
  </React.StrictMode>
);