import { useState, useEffect, useCallback, useRef } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';
import { Stack } from '@astryxdesign/core/Stack';
import { Heading } from '@astryxdesign/core/Heading';
import { Text } from '@astryxdesign/core/Text';
import { Button } from '@astryxdesign/core/Button';
import { Spinner } from '@astryxdesign/core/Spinner';
import { TextInput } from '@astryxdesign/core/TextInput';
import { TextArea } from '@astryxdesign/core/TextArea';
import { Banner } from '@astryxdesign/core/Banner';
import { Avatar } from '@astryxdesign/core/Avatar';
import { AvatarGroup,AvatarGroupOverflow } from '@astryxdesign/core/AvatarGroup';
import { SideNav, SideNavItem, SideNavSection } from '@astryxdesign/core/SideNav';
import { Dialog } from '@astryxdesign/core/Dialog';
import { ContextMenu } from '@astryxdesign/core/ContextMenu';
import { Home, Hash, FileText, CircleDot, Users, Plus, LogOut, ChevronRight, ChevronUp, LockKeyhole, Layers, PanelLeft, Settings as SettingsIcon, Moon, Sun, UserRoundPlus, Check } from 'lucide-react';
import { api } from '../lib/api';
import { IssueBoard } from './IssueBoard';
import { NotesView } from './NotesView';
import { Chat } from './Chat';
import { Markdown } from './Markdown';
import { Selector } from '@astryxdesign/core/Selector';
import { Switch } from '@astryxdesign/core/Switch';
import {PopupMenu} from './PopupMenu';
import {uploadFile} from '../lib/files';
import {requestNotifications,notify,enableSound} from '../lib/notify';

const errorMessage=e=>e.response?.data?.message||e.message||'Something went wrong.';
const slugify=s=>s.toLowerCase().trim().replace(/[^a-z0-9]+/g,'-').replace(/^-+|-+$/g,'')||'workspace';
const icons={issues:CircleDot,text:Hash,general:Hash,page:FileText,notes:FileText};
function Menu({items,children}){return <ContextMenu label="Actions" presentation="adaptive" items={items}>{children}</ContextMenu>}

export function Workspace({apiWarning,checkApi,user,onSignOut,onProfile,onSwitch,onAddAccount,accounts,theme,setTheme}) {
 const navigate=useNavigate(),location=useLocation();
 const match=location.pathname.match(/^\/team\/([^/]+)(?:\/channel\/([^/]+))?/); const teamId=match?.[1],channelId=match?.[2];
 const [teams,setTeams]=useState([]),[channels,setChannels]=useState([]),[members,setMembers]=useState([]),[error,setError]=useState(''),[dialog,setDialog]=useState(null),[name,setName]=useState(''),[email,setEmail]=useState(''),[kind,setKind]=useState('issues'),[busy,setBusy]=useState(false),[mobileNav,setMobileNav]=useState(false),[target,setTarget]=useState(null),[chosen,setChosen]=useState(null),[expanded,setExpanded]=useState({}),[teamSpaces,setTeamSpaces]=useState({}),[unread,setUnread]=useState([]),[accountOpen,setAccountOpen]=useState(false),[settingsOpen,setSettingsOpen]=useState(false),[popup,setPopup]=useState(null),[spaceTeam,setSpaceTeam]=useState(null),[alerts,setAlerts]=useState([]),[privateSpace,setPrivateSpace]=useState(true),[teamPhoto,setTeamPhoto]=useState(null),[teamsReady,setTeamsReady]=useState(false),[channelsFor,setChannelsFor]=useState(null),[draggedChannel,setDraggedChannel]=useState(null),[terminalOpen,setTerminalOpen]=useState(false),[terminalOutput,setTerminalOutput]=useState(''),[terminalInput,setTerminalInput]=useState(''),[terminalBusy,setTerminalBusy]=useState(false);
 const loadTeams=useCallback(async()=>{try{const r=await api.get('/api/teams');setTeams(r.data);}catch(e){setError(errorMessage(e));}finally{setTeamsReady(true);}},[]);
 const loadTeam=useCallback(async()=>{if(!teamId){setChannels([]);setMembers([]);setChannelsFor(null);return;}setChannelsFor(null);try{const [c,m]=await Promise.all([api.get('/api/channels',{params:{team_id:teamId}}),api.get(`/api/teams/${teamId}/members`)]);setChannels(c.data);setTeamSpaces(prev=>({...prev,[teamId]:c.data}));setMembers(m.data);}catch(e){setError(errorMessage(e));}finally{setChannelsFor(teamId);}},[teamId]);
 useEffect(()=>{const unlock=()=>enableSound();window.addEventListener('pointerdown',unlock,{once:true});window.addEventListener('keydown',unlock,{once:true});return()=>{window.removeEventListener('pointerdown',unlock);window.removeEventListener('keydown',unlock);};},[]);
 useEffect(()=>{if(!accountOpen)return;const close=e=>{if(e.type==='keydown'&&e.key!=='Escape')return;if(e.target?.closest?.('.account-footer'))return;setAccountOpen(false);};document.addEventListener('pointerdown',close);document.addEventListener('keydown',close);return()=>{document.removeEventListener('pointerdown',close);document.removeEventListener('keydown',close);};},[accountOpen]);
 useEffect(()=>{loadTeams();},[loadTeams]);useEffect(()=>{loadTeam();},[loadTeam]);
 const loadUnread=useCallback(()=>api.get('/api/channels/unread').then(r=>setUnread(r.data)).catch(()=>{}),[]);

  const reorderChannels=useCallback(async()=>{if(!teamId)return;const ordered=channels.map((c,i)=>({id:c.id,position:i}));for(const ch of ordered){try{await api.put(`/api/channels/${ch.id}/reorder`,{channel_id:ch.id,position:ch.position});}catch(e){console.error('Reorder failed',e);}}},[teamId,channels]);

  const onDragStart=(e,id)=>{setDraggedChannel(id);e.dataTransfer.effectAllowed='move';e.dataTransfer.setData('text/id',id);};
  const onDragOver=(e)=>{e.preventDefault();e.dataTransfer.dropEffect='move';};
  const onDrop=(e,targetId)=>{e.preventDefault();if(!draggedChannel||draggedChannel===targetId)return;const draggedIdx=channels.findIndex(c=>c.id===draggedChannel);const targetIdx=channels.findIndex(c=>c.id===targetId);if(draggedIdx<0||targetIdx<0)return;const newChannels=[...channels];const [moved]=newChannels.splice(draggedIdx,1);newChannels.splice(targetIdx,0,moved);setChannels(newChannels);setDraggedChannel(null);setTimeout(reorderChannels,100);};

  const channelIcons=c=>{if(c.type==='issues')return <FileText size={15}/>;if(c.type==='notes')return <FileText size={15}/>;if(c.type==='page')return <FileText size={15}/>;return c.is_private?<LockKeyhole size={15}/>:<Hash size={15}/>;};
 useEffect(()=>{loadUnread();const timer=setInterval(loadUnread,4000);window.addEventListener('boron-unread-changed',loadUnread);return()=>{clearInterval(timer);window.removeEventListener('boron-unread-changed',loadUnread);};},[loadUnread]);
 useEffect(()=>{let active=true;async function check(){try{const r=await api.get('/api/reminders');if(!active)return;for(const item of r.data){if(new Date(item.remind_at).getTime()>Date.now())continue;const ack=await api.post(`/api/reminders/${item.id}/delivered`);if(!active||!ack.data.delivered)continue;setAlerts(old=>[...old,item]);notify(`Reminder in #${item.channel_name}`,item.content||'Message reminder',()=>navigate(`/team/${item.team_id}/channel/${item.channel_id}?message=${item.message_id}`));}}catch{}}check();const timer=setInterval(check,5000);return()=>{active=false;clearInterval(timer);};},[user.id,navigate]);
 useEffect(()=>{if(teamId)setExpanded(prev=>({...prev,[teamId]:true}));},[teamId]);
 const team=teams.find(t=>t.id===teamId), channel=channels.find(c=>c.id===channelId);
 const isAdmin=team?.owner_id===user.id||members.some(m=>m.id===user.id&&['owner','admin'].includes(m.role));
 const open=(what)=>{if(what==='channel')setSpaceTeam(teamId);setName('');setEmail('');setChosen(null);setKind('issues');setPrivateSpace(true);setTeamPhoto(null);setError('');setDialog(what);};
 const createSpace=(id)=>{open('channel');setKind('text');setSpaceTeam(id);};
 const goTeam=id=>{setExpanded(prev=>({...prev,[id]:true}));navigate(`/team/${id}`);setMobileNav(false);};
 const goChannel=(id,tid=teamId)=>{navigate(`/team/${tid}/channel/${id}`);setMobileNav(false);};
 async function submit(e){e.preventDefault();setBusy(true);setError('');try{
  if(dialog==='team'){const r=await api.post('/api/teams',{name:name.trim(),slug:slugify(name)});if(teamPhoto){try{await uploadFile(`/api/teams/${r.data.id}/avatar`,teamPhoto);}catch(e){setError('Team created, but its picture could not be uploaded. You can add it in team settings.');}}await loadTeams();goTeam(r.data.id);}
  if(dialog==='channel'){const id=spaceTeam||teamId;const r=await api.post('/api/channels',{team_id:id,name:name.trim(),slug:slugify(name),type:kind,is_private:kind==='text'?privateSpace:true});if(id===teamId)await loadTeam();goChannel(r.data.id,id);}
  if(dialog==='team-settings'){if(teamPhoto)await uploadFile(`/api/teams/${teamId}/avatar`,teamPhoto);await loadTeams();}
  if(dialog==='team-member'){await api.post(`/api/teams/${teamId}/members`,chosen?{user_id:chosen.id}:{username:email.trim()});await loadTeam();}
  if(dialog==='channel-member'){await api.post(`/api/channels/${channelId}/members`,chosen?{user_id:chosen.id}:{username:email.trim()});await loadTeam();window.dispatchEvent(new Event('boron-members-changed'));}
  if(dialog==='delete-channel'){await api.delete(`/api/channels/${target.id}`);await loadTeam();if(channelId===target.id)goTeam(teamId);}
  setDialog(null);
 }catch(e){setError(errorMessage(e));}finally{setBusy(false);}}
 const header=<Stack direction="horizontal" gap={2} vAlign="center" padding={3}><Avatar name={team?.name||'Boron'} src={team?.avatar_url||undefined} size="sm"/><Stack size="fill"><Text weight="semibold">{team?.name||'Boron workspace'}</Text><Text size="xs" type="supporting">Workspace</Text></Stack></Stack>;
  const footer=(
    <Stack className="account-footer">
      <button type="button" className={`account-button ${accountOpen?'account-button-open':''}`} aria-label="Account menu" aria-haspopup="dialog" aria-expanded={accountOpen} aria-controls="account-menu" onClick={()=>{enableSound();setAccountOpen(v=>!v);}}>
        <Avatar name={user.display_name} src={user.avatar_url||undefined} size="sm"/>
        <span className="account-summary"><strong>{user.display_name}</strong><small>{user.email}</small></span>
        <ChevronUp className={`account-chevron ${accountOpen?'open':''}`} size={16}/>
      </button>
      {terminalOpen&&<TerminalDialog user={user} onClose={()=>setTerminalOpen(false)}/>}
      {accountOpen&&(
        <Stack id="account-menu" role="dialog" aria-label="Account" className="account-menu" gap={0}>
          <Stack direction="horizontal" gap={2} vAlign="center" className="account-menu-profile">
            <Avatar name={user.display_name} src={user.avatar_url||undefined} size="sm"/>
            <Stack className="account-profile-text" gap={0}>
              <Text weight="semibold" size="sm">{user.display_name}</Text>
              <Text size="xs" type="supporting">{user.email}</Text>
            </Stack>
            <Check size={15}/>
          </Stack>
          <Stack className="account-menu-section" gap={0}>
            <button type="button" onClick={()=>{setAccountOpen(false);setSettingsOpen(true);}}>
              <SettingsIcon size="16"/><span>Settings</span>
            </button>
            {isAdmin&&(
              <button type="button" onClick={()=>{setAccountOpen(false);setTerminalOpen(true);}}>
                <PanelLeft size="16"/><span>Admin Terminal</span>
              </button>
            )}
            <Stack className="account-theme-row" direction="horizontal" vAlign="center">
              <Sun size="16"/><span>Appearance</span>
              <Switch label="Dark mode" isLabelHidden size="sm" value={theme==='dark'} onChange={value=>setTheme(value?'dark':'light')}/>
              <Moon size="16"/>
            </Stack>
            {accounts.filter(a=>a.user.id!==user.id).length>0&&(
              <Stack className="account-menu-section" gap={0}>
                <Text className="account-section-label" size="xs" type="supporting">SWITCH ACCOUNT</Text>
                {accounts.filter(a=>a.user.id!==user.id).map(a=>
                  <button type="button" key={a.user.id} onClick={()=>onSwitch(a)}>
                    <Avatar name={a.user.display_name} src={a.user.avatar_url||undefined} size="xsm"/>
                    <span>Switch to {a.user.display_name}</span>
                  </button>
                )}
              </Stack>
            )}
            <Stack className="account-menu-section" gap={0}>
              <button type="button" aria-label="Add / switch account" onClick={onAddAccount}>
                <UserRoundPlus size="16"/><span>Add account</span>
              </button>
              <button type="button" className="destructive" onClick={onSignOut}>
                <LogOut size="16"/><span>Sign out</span>
              </button>
            </Stack>
          </Stack>
        </Stack>
      )}
    </Stack>
  );
 return <Stack direction="horizontal" className="workspace" height="100%" width="100%">
  <aside onContextMenu={e=>{if(e.target.closest('.astryx-side-nav-item,button,a'))return;if(teamId&&isAdmin){e.preventDefault();setPopup({x:e.clientX,y:e.clientY,items:[{label:'Create channel',onClick:()=>createSpace(teamId)}]});}}} className={`sidebar ${mobileNav?'sidebar-open':''}`}><SideNav header={header} footer={footer}>
   <SideNavSection title="Workspace"><SideNavItem label="Home" icon={<Home size={16}/>} isSelected={!teamId} onClick={()=>{navigate('/');setMobileNav(false);}}/><SideNavItem label="New team" icon={<Plus size={16}/>} onClick={()=>open('team')}/></SideNavSection>
   <SideNavSection title="Your teams">{teams.map(t=>{
    const teamCount=unread.filter(u=>u.team_id===t.id).reduce((sum,u)=>sum+u.count,0);
    const current=t.id===teamId;
    return <Menu key={t.id} items={[{label:'Open team',onClick:()=>goTeam(t.id)},...(['owner','admin'].includes(t.role)||(current&&isAdmin)?[{label:'Create channel',onClick:()=>createSpace(t.id)},...(current?[{label:'Team settings',onClick:()=>open('team-settings')}]:[])]:[])]}><SideNavItem label={t.name} icon={<Layers size={16}/>} isSelected={current&&!channelId} onClick={()=>goTeam(t.id)}
      endContent={teamCount>0?<Text className="team-unread" size="xs">{teamCount}</Text>:undefined}
      actions={current&&isAdmin?<Button label="Create space" isIconOnly icon={<Plus size={15}/>} size="sm" variant="ghost" onClick={()=>open('channel')}/>:undefined}
      collapsible={{isCollapsed:!expanded[t.id],onCollapsedChange:collapsed=>{setExpanded(prev=>({...prev,[t.id]:!collapsed}));if(!collapsed&&!teamSpaces[t.id])api.get('/api/channels',{params:{team_id:t.id}}).then(r=>setTeamSpaces(prev=>({...prev,[t.id]:r.data}))).catch(e=>setError(errorMessage(e)));}}}>
      {(teamSpaces[t.id]||[]).map(c=>{const Icon=icons[c.type]||Hash;const n=unread.find(u=>u.channel_id===c.id)?.count||0;const isPage=c.type==='page'||c.type==='notes';return <div key={c.id} onContextMenu={e=>e.stopPropagation()}><Menu items={[{label:isPage?'Open page':'Open channel',onClick:()=>goChannel(c.id,t.id)},...(current&&isAdmin?[{label:'Add a person',onClick:()=>{goChannel(c.id);open('channel-member');}},{label:isPage?'Delete page':'Delete channel',variant:'destructive',onClick:()=>{setTarget(c);open('delete-channel');}}]:[])]}><SideNavItem label={c.name} icon={<NoticeIcon Icon={Icon} count={n}/>} isSelected={channelId===c.id} onClick={()=>goChannel(c.id,t.id)}/></Menu></div>})}
      {teamSpaces[t.id]?.length===0&&<SideNavItem label="No spaces" isDisabled/>}
    </SideNavItem></Menu>;
   })}</SideNavSection>
  </SideNav></aside>
  <main className="main-panel"><header className="topbar"><Stack direction="horizontal" gap={2} vAlign="center"><Button label="Toggle navigation" variant="ghost" size="sm" isIconOnly icon={<PanelLeft size={17}/>} className="mobile-toggle" onClick={()=>setMobileNav(v=>!v)}/><Text weight="semibold">{team?.name||'Workspace'}</Text>{channel&&<><ChevronRight size={14}/><Text weight="semibold">{channel.name}</Text></>}</Stack><Stack direction="horizontal" gap={2} vAlign="center">{team&&isAdmin&&<Button label={channel?'Add people':'Team members'} size="sm" variant="ghost" icon={<Users size={16}/>} onClick={()=>open(channel?'channel-member':'team-member')}/>}{!team&&<Button label="New team" size="sm" variant="primary" icon={<Plus size={16}/>} onClick={()=>open('team')}/>}</Stack></header>
   {alerts.length>0&&<div className="reminder-alert" role="status">{alerts.map(a=><div key={a.id}><button type="button" onClick={()=>{navigate(`/team/${a.team_id}/channel/${a.channel_id}?message=${a.message_id}`);setAlerts(v=>v.filter(x=>x.id!==a.id));}}>Reminder in #{a.channel_name}: {a.content||'Message'}</button><button type="button" aria-label="Dismiss reminder" onClick={()=>setAlerts(v=>v.filter(x=>x.id!==a.id))}>×</button></div>)}</div>}
   {apiWarning&&<Stack padding={3}><Banner status="warning" title="Backend connection check" description={apiWarning} endContent={<Button label="Retry connection" size="sm" variant="ghost" onClick={()=>checkApi()}/>}/></Stack>}
   {error&&!dialog&&<Stack padding={3}><Banner status="error" title={error}/></Stack>}
   <section className="content-region" key={`${teamId||'home'}-${channelId||''}`}>
    {!teamsReady?<Stack padding={5} hAlign="center"><Spinner label="Loading workspace…" size="lg"/></Stack>:!teamId?<HomePage user={user} teams={teams} openTeam={goTeam} create={()=>open('team')}/>:!team?<Stack padding={5}><Text>Team unavailable or you do not have access.</Text></Stack>:channelsFor!==teamId?<Stack padding={5} hAlign="center"><Spinner label="Loading team…" size="lg"/></Stack>:!channelId?<TeamHome team={team} channels={channels} members={members} isAdmin={isAdmin} create={()=>open('channel')} invite={()=>open('team-member')} settings={()=>open('team-settings')} go={goChannel}/>:!channel?<Stack padding={5}><Text>This channel is unavailable or you do not have access.</Text></Stack>:channel.type==='issues'?<IssueBoard teamId={teamId} channel={channel} members={members}/>:channel.type==='page'?<PageView channel={channel}/>:channel.type==='notes'?<NotesView channel={channel} teamId={teamId}/>:<Chat channel={channel} user={user} isAdmin={isAdmin} theme={theme}/>}
   </section>
  </main>
  <PopupMenu menu={popup} onClose={()=>setPopup(null)}/>
  {settingsOpen&&<Settings user={user} theme={theme} setTheme={setTheme} onClose={()=>setSettingsOpen(false)} onProfile={onProfile}/>}
  <Dialog isOpen={!!dialog} onOpenChange={v=>{if(!v)setDialog(null);}} width={440} aria-label="Workspace action"><form onSubmit={submit}><Stack gap={4}><Heading level={2}>{dialog==='team'?'Create a team':dialog==='channel'?'Create a space':dialog==='team-settings'?'Team settings':dialog==='channel-member'?'Add to this space':dialog==='delete-channel'?`Delete ${target?.name}?`:'Add a team member'}</Heading>{dialog==='delete-channel'&&<Text type="supporting">This permanently deletes this space and its contents.</Text>}{error&&<Banner status="error" title={error}/>}{(dialog==='team'||dialog==='channel')&&<TextInput label="Name" value={name} onChange={setName} required/>}{(dialog==='team'||dialog==='team-settings')&&<label className="field-label">Team picture<input type="file" accept="image/png,image/jpeg,image/gif,image/webp" onChange={e=>setTeamPhoto(e.target.files?.[0]||null)}/></label>}{dialog==='channel'&&<Selector label="Type" value={kind} onChange={setKind} options={[{value:"issues",label:"Issue board"},{value:"text",label:"Chat channel"},{value:"page",label:"Page"}]}/>}{dialog==='channel'&&kind==='text'&&<><Selector label="Visibility" value={privateSpace?'private':'public'} onChange={v=>setPrivateSpace(v==='private')} options={[{value:'private',label:'Private'},{value:'public',label:'Public'}]}/><Text size="xs" type="supporting">{privateSpace?'Only explicitly added people and team admins can access this channel.':'Every team member can access this channel automatically.'}</Text></>}{dialog==='channel'&&kind!=='text'&&<Text size="xs" type="supporting">Issue boards and pages are private.</Text>}{(dialog==='team-member'||dialog==='channel-member')&&<InviteSearch teamId={teamId} value={email} onChange={v=>{setEmail(v);setChosen(null);}} selected={chosen} onSelect={person=>{setChosen(person);setEmail(person.display_name);}}/>}<Stack direction="horizontal" gap={2} hAlign="end"><Button label="Cancel" variant="ghost" onClick={()=>setDialog(null)}/><Button label={dialog?.includes('member')?'Add person':dialog==='delete-channel'?'Delete':dialog==='team-settings'?'Save':'Create'} type="submit" variant="primary" isLoading={busy}/></Stack></Stack></form></Dialog>
 </Stack>;
}
function HomePage({user,teams,openTeam,create}){const [people,setPeople]=useState({});useEffect(()=>{let live=true;Promise.all(teams.map(t=>api.get(`/api/teams/${t.id}/members`).then(r=>[t.id,r.data]).catch(()=>[t.id,[]]))).then(rows=>{if(live)setPeople(Object.fromEntries(rows));});return()=>{live=false;};},[teams]);return <Stack className="home-content" gap={5}><Stack gap={2}><Text size="sm" type="supporting">YOUR WORKSPACE</Text><Heading level={1}>Good to see you, {user.display_name}.</Heading></Stack><Stack gap={2}><Stack direction="horizontal" hAlign="between" vAlign="center"><Heading level={2} size="sm">Your teams</Heading><Button label="New team" variant="ghost" size="sm" icon={<Plus size={15}/>} onClick={create}/></Stack>{teams.length===0?<Stack className="empty-box" gap={2}><Text>No teams yet.</Text><Text size="sm" type="supporting">Create one to start bringing your work together.</Text><Button label="Create your first team" variant="primary" onClick={create}/></Stack>:<div className="team-grid">{teams.map((t,i)=>{const teamPeople=people[t.id]||[];return <button type="button" key={t.id} className={`team-tile team-tile-${parseInt(t.id.slice(0,2),16)%4}`} onClick={()=>openTeam(t.id)}><Avatar name={t.name} src={t.avatar_url||undefined} size="lg"/><strong>{t.name}</strong><span className="team-tile-members">{teamPeople.length>0&&<AvatarGroup size="sm">{teamPeople.slice(0,3).map(m=><Avatar key={m.id} name={m.display_name} src={m.avatar_url||undefined}/>)}{t.member_count>3&&<AvatarGroupOverflow count={t.member_count-3}/>}</AvatarGroup>}<span>{t.member_count} {t.member_count===1?'member':'members'}</span></span></button>})}</div>}</Stack></Stack>}
function TeamHome({team,channels,members,isAdmin,create,invite,settings,go}){return <Stack className="home-content" gap={5}><Stack gap={2}><Text size="sm" type="supporting">TEAM OVERVIEW</Text><Heading level={1}>{team.name}</Heading><Text type="supporting">{members.length} {members.length===1?'member':'members'} · {channels.length} spaces</Text></Stack><Stack direction="horizontal" gap={2}>{isAdmin&&<Button label="Create a space" variant="primary" icon={<Plus size={16}/>} onClick={create}/>} {isAdmin&&<Button label="Add team member" variant="secondary" icon={<Users size={16}/>} onClick={invite}/>} {isAdmin&&<Button label="Team settings" variant="secondary" icon={<SettingsIcon size={16}/>} onClick={settings}/>}</Stack><Stack gap={2}><Heading level={2} size="sm">Spaces</Heading>{channels.length===0?<Stack className="empty-box" gap={2}><Text>No spaces yet.</Text></Stack>:channels.map(c=><Button key={c.id} label={c.name} variant="secondary" icon={channelIcons(c)} onClick={()=>go(c.id)} draggable onDragStart={(e)=>onDragStart(e,c.id)} onDragOver={onDragOver} onDrop={(e)=>onDrop(e,c.id)} style={{cursor:"grab",opacity:draggedChannel===c.id?0.5:1}}/>)}</Stack></Stack>}
function PageView({channel}){const [content,setContent]=useState(''),[editing,setEditing]=useState(false),[error,setError]=useState(''),[saved,setSaved]=useState(false),[loading,setLoading]=useState(true);useEffect(()=>{api.get(`/api/channels/${channel.id}/page`).then(r=>setContent(r.data.content)).catch(e=>setError(errorMessage(e))).finally(()=>setLoading(false));},[channel.id]);async function save(){try{await api.put(`/api/channels/${channel.id}/page`,{content});setEditing(false);setSaved(true);}catch(e){setError(errorMessage(e));}}return <Stack className="page-view" gap={4}><Stack direction="horizontal" hAlign="between" vAlign="center"><Stack gap={1}><Text size="sm" type="supporting">PRIVATE PAGE</Text><Heading level={1}>{channel.name}</Heading></Stack><Button label={editing?'Save page':'Edit page'} variant={editing?'primary':'secondary'} onClick={()=>editing?save():setEditing(true)}/></Stack>{error&&<Banner status="error" title={error}/>}{loading?<Spinner label="Loading page…" size="lg"/>:editing?<TextArea label="Page content (Markdown)" value={content} onChange={setContent} rows={16}/>:<article className="markdown">{content?<Markdown>{content}</Markdown>:<Text type="supporting">This page is empty. Add some Markdown to get started.</Text>}</article>}{saved&&<Text size="xs" type="supporting">Changes saved</Text>}</Stack>}

function TerminalDialog({user,onClose}){
  const [output,setOutput]=useState("Welcome to Boron Admin Terminal\nType 'help' for available commands.\n");
  const [input,setInput]=useState('');
  const [busy,setBusy]=useState(false);
  const isAdmin=user.role==='admin'||user.role==='owner';
  const run=async()=>{if(!input.trim()||!isAdmin)return;setBusy(true);try{const r=await api.post('/api/admin/command',{command:input.trim()});setOutput(prev=>prev+'\\n$ '+input.trim()+'\\n'+(r.data.output||''));setInput('');}catch(e){setOutput(prev=>prev+'\\nError: '+(e.response?.data?.message||e.message));}finally{setBusy(false);}};
  return <Dialog isOpen onOpenChange={v=>{if(!v)onClose();}} width={600} aria-label="Admin Terminal"><Stack gap={3}><Heading level={2}>Admin Terminal</Heading><Stack className="terminal-wrap" height={300}><pre className="terminal-output">{output}</pre></Stack><form onSubmit={e=>{e.preventDefault();run();}}><Stack direction="horizontal" gap={2}><span className="terminal-prompt">$</span><input type="text" value={input} onChange={e=>setInput(e.target.value)} onKeyDown={e=>{if(e.key==='Enter')run();}} disabled={busy||!isAdmin} placeholder={isAdmin?'Enter command (help for list)':'Admin access required'}/>{busy&&<Spinner size="sm"/>}</Stack></form>{!isAdmin&&<Banner status="error" title="Admin access required."/>}</Stack></Dialog>;
}

function InviteSearch({teamId,value,onChange,selected,onSelect}){
 const [items,setItems]=useState([]),[open,setOpen]=useState(false);
 useEffect(()=>{if(selected||value.trim().length<2){setItems([]);return;}let active=true;const t=setTimeout(()=>api.get(`/api/teams/${teamId}/user-search`,{params:{q:value}}).then(r=>{if(active){setItems(r.data);setOpen(true);}}).catch(()=>{if(active)setItems([]);}),180);return()=>{active=false;clearTimeout(t);};},[teamId,value,selected]);
 return <Stack gap={2} className="invite-search"><TextInput label="Username or email" value={value} onChange={onChange} autoComplete="off"/>{open&&items.length>0&&!selected&&<Stack className="suggestions" role="listbox" aria-label="People suggestions">{items.map(person=><button type="button" className="suggestion-row" key={person.id} onClick={()=>{onSelect(person);setOpen(false);}}><Avatar name={person.display_name} src={person.avatar_url||undefined} size="sm"/><Stack gap={0}><Text>{person.display_name}</Text><Text size="xs" type="supporting">{person.email}</Text></Stack></button>)}</Stack>}</Stack>;
}

function NoticeIcon({Icon,count}){return <Stack className="notice-icon"><Icon size={16}/>{count>0&&<Text className="notice-count" size="xs" aria-label={`${count} unread mentions`}>{count>99?'99+':count}</Text>}</Stack>}

function Settings({user,theme,setTheme,onClose,onProfile}){
 const [name,setName]=useState(user.display_name),[bio,setBio]=useState(user.bio||''),[file,setFile]=useState(null),[err,setErr]=useState(''),[busy,setBusy]=useState(false),[permission,setPermission]=useState(typeof Notification==='undefined'?'unsupported':Notification.permission);
 async function save(e){e.preventDefault();setBusy(true);setErr('');try{let r=await api.put('/api/auth/me',{display_name:name,bio});let profile=r.data;if(file){const image=await uploadFile('/api/auth/avatar',file);profile={...profile,avatar_url:image.avatar_url};}onProfile(profile);onClose();}catch(e){setErr(errorMessage(e));}finally{setBusy(false);}}
 return <Dialog isOpen onOpenChange={v=>{if(!v)onClose();}} width={520} aria-label="Settings"><form onSubmit={save}><Stack gap={4}><Heading level={2}>Settings</Heading>{err&&<Banner status="error" title={err}/>}<Stack direction="horizontal" gap={3} vAlign="center"><Avatar name={name||user.display_name} src={user.avatar_url||undefined} size="lg"/><label className="field-label">Profile picture<input type="file" accept="image/png,image/jpeg,image/gif,image/webp" onChange={e=>setFile(e.target.files?.[0]||null)}/></label></Stack><TextInput label="Display name" value={name} onChange={setName} required/><TextArea label="Bio" value={bio} onChange={setBio} rows={3}/><Stack direction="horizontal" gap={2} vAlign="center"><Text>Theme: {theme}</Text><Button label="Toggle theme" variant="secondary" size="sm" onClick={()=>setTheme(theme==='dark'?'light':'dark')}/></Stack><Stack gap={1}><Button label="Enable notifications and sound" variant="secondary" onClick={async()=>setPermission(await requestNotifications())}/><Text size="xs" type="supporting">Notifications: {permission}. Reminders appear while Boron is open; overdue reminders appear when you return.</Text></Stack><Stack direction="horizontal" hAlign="end" gap={2}><Button label="Cancel" variant="ghost" onClick={onClose}/><Button label="Save settings" type="submit" variant="primary" isLoading={busy}/></Stack></Stack></form></Dialog>;
}
