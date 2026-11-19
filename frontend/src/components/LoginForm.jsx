import { useState } from 'react';
import { Stack } from '@astryxdesign/core/Stack';
import { Heading } from '@astryxdesign/core/Heading';
import { Text } from '@astryxdesign/core/Text';
import { Button } from '@astryxdesign/core/Button';
import { Card } from '@astryxdesign/core/Card';
import { Banner } from '@astryxdesign/core/Banner';
import { TextInput } from '@astryxdesign/core/TextInput';
import { api, setAuthToken } from '../lib/api';
export function LoginForm({ onLogin }) {
 const [email,setEmail]=useState(''),[password,setPassword]=useState(''),[error,setError]=useState(''),[busy,setBusy]=useState(false);
 async function submit(e){e.preventDefault();setBusy(true);setError('');try{const res=await api.post('/api/auth/login',{email,password});setAuthToken(res.data.token);onLogin(res.data.user);}catch(err){setError(err.response?.data?.message||'Invalid email or password.');}finally{setBusy(false);}}
 return <Stack padding={4} hAlign="center" vAlign="center" height="100%" width="100%"><Card width={400} padding={5}><Stack gap={4}><Stack gap={1}><Text size="sm" type="supporting">BORON WORKSPACE</Text><Heading level={2}>Welcome back</Heading><Text type="supporting">Sign in to pick up where you left off.</Text></Stack>{error&&<Banner status="error" title={error}/>}<form onSubmit={submit}><Stack gap={4}><TextInput label="Email" type="email" value={email} onChange={setEmail} autoComplete="username" required/><TextInput label="Password" type="password" value={password} onChange={setPassword} autoComplete="current-password" required/><Button label="Sign in" variant="primary" type="submit" width="100%" isLoading={busy}/></Stack></form><Stack direction="horizontal" gap={2} hAlign="center" vAlign="center"><Text size="sm" type="supporting">New to Boron?</Text><Button label="Create account" variant="ghost" href="/register" size="sm"/></Stack></Stack></Card></Stack>;
}
