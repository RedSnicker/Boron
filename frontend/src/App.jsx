import { useState, useEffect } from 'react';
import { Routes, Route, Navigate, useNavigate } from 'react-router-dom';
import { Stack } from '@astryxdesign/core/Stack';
import { Text } from '@astryxdesign/core/Text';
import { Button } from '@astryxdesign/core/Button';
import { Spinner } from '@astryxdesign/core/Spinner';
import { api, setAuthToken, getAuthToken } from './lib/api';
import { LoginForm } from './components/LoginForm';
import { RegisterForm } from './components/RegisterForm';
import { Workspace } from './components/Workspace';
import './index.css';
const accounts=()=>{try{return JSON.parse(localStorage.getItem('boron_accounts')||'[]')}catch{return []}};
function remember(user){const token=getAuthToken();if(token&&user){const list=accounts().filter(a=>a.user.id!==user.id);localStorage.setItem('boron_accounts',JSON.stringify([...list,{user,token}]));}}
export default function App({theme,setTheme}){
 const [user,setUser]=useState(null),[loading,setLoading]=useState(!!getAuthToken()),[offline,setOffline]=useState(false),[apiWarning,setApiWarning]=useState(false); const navigate=useNavigate();
 useEffect(()=>{const expire=()=>{setUser(null);setOffline(false);navigate('/login',{replace:true});};window.addEventListener('boron-session-expired',expire);return()=>window.removeEventListener('boron-session-expired',expire);},[navigate]);
 useEffect(()=>{if(!getAuthToken()){setLoading(false);return;}api.get('/api/auth/me').then(res=>{setUser(res.data);remember(res.data);}).catch(err=>{if(err.response?.status!==401)setOffline(true);}).finally(()=>setLoading(false));},[]);
 async function checkApi(routeIssue){try{const r=await api.get('/api/version');const rev=Number(r.data?.api_revision);if(r.data?.service!=='boron-backend'||!Number.isFinite(rev)){setApiWarning('The API version response was not recognized. Check where /api is being routed.');return;}if(rev<7){setApiWarning(`The server handling /api/version reports revision ${rev}; this frontend needs revision 7 or newer. Check the backend process and proxy target.`);return;}setApiWarning(routeIssue?`The version endpoint reports revision ${rev}, but /api/channels/unread returned 405. Check that both routes reach the same backend.`:false);}catch(e){if([404,405].includes(e.response?.status))setApiWarning(`/api/version returned ${e.response.status}. Check the running backend and proxy target.`);else setApiWarning('Could not verify /api/version. Check the backend connection and retry.');}}
 useEffect(()=>{if(!user){setApiWarning(false);return;}checkApi();const report=()=>checkApi(true);window.addEventListener('boron-api-incompatible',report);return()=>window.removeEventListener('boron-api-incompatible',report);},[user?.id]);
 useEffect(()=>{if(!user||!apiWarning||apiWarning.includes('/api/channels/unread'))return;const timer=setInterval(()=>checkApi(),15000);return()=>clearInterval(timer);},[user?.id,apiWarning]);
 const signedIn=u=>{setUser(u);remember(u);navigate('/',{replace:true});};
 if(loading)return <Stack padding={4} hAlign="center" vAlign="center" height="100%"><Spinner label="Restoring your session…" size="lg"/></Stack>;
 if(offline)return <Stack padding={4} gap={3} hAlign="center" vAlign="center" height="100%"><Text>Could not restore your session. Check your connection and try again.</Text><Button label="Retry" onClick={()=>window.location.reload()}/><Button label="Sign out" variant="ghost" onClick={()=>{setAuthToken(null);setOffline(false);}}/></Stack>;
 if(!user)return <Routes><Route path="/register" element={<RegisterForm onRegister={signedIn}/>}/><Route path="/login" element={<LoginForm onLogin={signedIn}/>}/><Route path="*" element={<Navigate to="/login" replace/>}/></Routes>;
 return <Workspace key={user.id} apiWarning={apiWarning} checkApi={checkApi} user={user} theme={theme} setTheme={setTheme} accounts={accounts()} onProfile={u=>{setUser(u);remember(u);}} onSwitch={async a=>{setAuthToken(a.token);try{const r=await api.get('/api/auth/me');setUser(r.data);remember(r.data);navigate('/');}catch{localStorage.setItem('boron_accounts',JSON.stringify(accounts().filter(x=>x.user.id!==a.user.id)));setAuthToken(null);setUser(null);navigate('/login');}}} onAddAccount={()=>{setAuthToken(null);setUser(null);navigate('/login');}} onSignOut={()=>{localStorage.setItem('boron_accounts',JSON.stringify(accounts().filter(a=>a.user.id!==user.id)));setAuthToken(null);setUser(null);navigate('/login',{replace:true});}}/>;
}
