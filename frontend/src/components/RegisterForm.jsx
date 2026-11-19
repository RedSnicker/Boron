import { useState, useEffect, useRef } from 'react';
import { Stack } from '@astryxdesign/core/Stack';
import { Heading } from '@astryxdesign/core/Heading';
import { Text } from '@astryxdesign/core/Text';
import { Button } from '@astryxdesign/core/Button';
import { Card } from '@astryxdesign/core/Card';
import { Banner } from '@astryxdesign/core/Banner';
import { TextInput } from '@astryxdesign/core/TextInput';
import { Avatar } from '@astryxdesign/core/Avatar';
import { Camera, ImagePlus } from 'lucide-react';
import { api, setAuthToken } from '../lib/api';
import { MAX_FILE, uploadFile } from '../lib/files';

const tones=['purple','teal','orange','pink'];
const initials=name=>{const parts=name.trim().split(/\s+/u).filter(Boolean);return (parts.length>1?Array.from(parts[0])[0]+Array.from(parts.at(-1))[0]:Array.from(parts[0]||'').slice(0,2).join('')).toLocaleUpperCase()||'YOU';};
function presetFile(choice,name){
 return new Promise((resolve,reject)=>{try{
  const probe=document.createElement('span');probe.style.backgroundColor=`var(--color-background-${choice.tone})`;probe.style.color=`var(--color-text-${choice.tone})`;document.body.appendChild(probe);const style=getComputedStyle(probe),bg=style.backgroundColor,fg=style.color;probe.remove();
  const canvas=document.createElement('canvas');canvas.width=512;canvas.height=512;const ctx=canvas.getContext('2d');ctx.fillStyle=bg;ctx.fillRect(0,0,512,512);ctx.textAlign='center';ctx.textBaseline='middle';ctx.fillStyle=fg;ctx.font='600 184px system-ui, sans-serif';ctx.fillText(initials(name),256,270,430);
  canvas.toBlob(blob=>blob?resolve(new File([blob],'profile.png',{type:'image/png'})):reject(new Error('Could not create profile image.')),'image/png');
 }catch(e){reject(e);}});
}
export function RegisterForm({ onRegister }) {
 const [step,setStep]=useState(0),[email,setEmail]=useState(''),[password,setPassword]=useState(''),[name,setName]=useState(''),[choice,setChoice]=useState(null),[error,setError]=useState(''),[busy,setBusy]=useState(false),[tone]=useState(()=>tones[Math.floor(Math.random()*tones.length)]);
 const photo=useRef(null),camera=useRef(null),[avatarPreview,setAvatarPreview]=useState(null);
 useEffect(()=>{if(choice?.kind!=='photo'){setAvatarPreview(null);return;}const url=URL.createObjectURL(choice.file);setAvatarPreview(url);return()=>URL.revokeObjectURL(url);},[choice]);
 const picture=e=>{const file=e.target.files?.[0];e.target.value='';if(!file)return;if(file.size>MAX_FILE||!['image/png','image/jpeg','image/gif','image/webp'].includes(file.type)){setError('Choose a PNG, JPEG, GIF or WebP image under 800 MiB.');return;}setChoice({kind:'photo',file});setError('');};
 async function submit(e){e.preventDefault();setError('');if(step===0){if(!email.trim()||password.length<8){setError('Enter an email and a password of at least 8 characters.');return;}setStep(1);return;}if(!name.trim()){setError('Enter a profile username.');return;}setBusy(true);try{
  const avatar=choice?.kind==='photo'?choice.file:await presetFile({tone},name);
  const res=await api.post('/api/auth/register',{email:email.trim(),password,display_name:name.trim(),avatar_url:null});setAuthToken(res.data.token);let profile=res.data.user;
  if(avatar){try{const uploaded=await uploadFile('/api/auth/avatar',avatar);profile={...profile,avatar_url:uploaded.avatar_url};}catch(uploadError){window.alert('Account created, but the profile picture could not be uploaded.');}}
  onRegister(profile);
 }catch(err){setError(err.response?.data?.message||err.message||'Registration failed. Please try again.');}finally{setBusy(false);}}
 return <Stack padding={4} hAlign="center" vAlign="center" height="100%" width="100%" className="register-page"><Card width={400} padding={5} className="register-card">
  <Stack gap={4}><Stack gap={1}><Text size="sm" type="supporting">BORON · STEP {step+1} OF 2</Text><Heading level={2}>{step===0?'Create your account':'Make it yours'}</Heading><Text type="supporting">{step===0?'Start with your email and password.':'Choose a name and make your profile feel like you.'}</Text></Stack>
  {error&&<Banner status="error" title={error}/>}
  <form onSubmit={submit}><Stack gap={4}>{step===0?<><TextInput label="Email" type="email" value={email} onChange={setEmail} autoComplete="username" required/><TextInput label="Password" type="password" value={password} onChange={setPassword} autoComplete="new-password" required/></>:<>
   <Stack hAlign="center" className="profile-preview-wrap"><ProfilePreview name={name} choice={choice} url={avatarPreview} tone={tone}/></Stack>
   <TextInput label="Profile username" value={name} onChange={setName} autoComplete="name" required/>
   <Stack gap={2}><Text size="xs" weight="semibold" type="supporting">CHOOSE YOUR PHOTO</Text><Stack direction="horizontal" gap={2} className="profile-options">
    <ProfileOption label="Camera" icon={<Camera size={18}/>} onClick={()=>camera.current?.click()}/><ProfileOption label="Photos" icon={<ImagePlus size={18}/>} onClick={()=>photo.current?.click()}/>
   </Stack><input ref={photo} id="profile-picture" className="sr-only" type="file" accept="image/png,image/jpeg,image/gif,image/webp" aria-label="Choose profile picture" onChange={picture}/><input ref={camera} className="sr-only" type="file" accept="image/*" capture="user" aria-label="Take profile picture" onChange={picture}/>
   <Text size="xs" type="supporting">If you don’t choose a photo, a profile picture will be generated from your name.</Text></Stack>
  </>}
  <Button type="submit" label={step===0?'Continue':'Create account'} variant="primary" width="100%" isLoading={busy}/>{step===1&&<Button label="Back" variant="ghost" width="100%" onClick={()=>{setStep(0);setError('');}}/>}</Stack></form>
  <Stack direction="horizontal" gap={2} hAlign="center" vAlign="center"><Text size="sm" type="supporting">Already have an account?</Text><Button label="Sign in" variant="ghost" href="/login" size="sm"/></Stack></Stack>
 </Card></Stack>;
}
function ProfileOption({label,icon,onClick,selected}){return <button type="button" className="profile-option" aria-pressed={!!selected} onClick={onClick}><span className="profile-option-icon">{icon}</span><span>{label}</span></button>}
function ProfilePreview({name,choice,url,tone}){if(choice?.kind==='photo')return <Avatar name={name||'Your profile'} src={url||undefined} size={128}/>;return <Stack className={`profile-hero tone-${tone}`} hAlign="center" vAlign="center" aria-label="Generated profile preview">{initials(name)}</Stack>;}
