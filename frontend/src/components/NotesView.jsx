import {useState,useEffect,useCallback} from 'react';
import {Stack} from '@astryxdesign/core/Stack';
import {Heading} from '@astryxdesign/core/Heading';
import {Text} from '@astryxdesign/core/Text';
import {Button} from '@astryxdesign/core/Button';
import {Spinner} from '@astryxdesign/core/Spinner';
import {Card} from '@astryxdesign/core/Card';
import {Banner} from '@astryxdesign/core/Banner';
import {TextInput} from '@astryxdesign/core/TextInput';
import {TextArea} from '@astryxdesign/core/TextArea';
import {api} from '../lib/api';
export function NotesView({channel,teamId}){
 const [notes,setNotes]=useState([]),[title,setTitle]=useState(''),[content,setContent]=useState(''),[error,setError]=useState(''),[loading,setLoading]=useState(true);
 const load=useCallback(()=>api.get('/api/notes',{params:{channel_id:channel.id}}).then(r=>setNotes(r.data)).catch(e=>setError(e.response?.data?.message||'Could not load notes.')).finally(()=>setLoading(false)),[channel.id]);
 useEffect(()=>{load();},[load]);
 async function create(e){e.preventDefault();try{await api.post('/api/notes',{team_id:teamId,channel_id:channel.id,title,content});setTitle('');setContent('');load();}catch(e){setError(e.response?.data?.message||'Could not create note.');}}
 return <Stack className="page-view" gap={4}><Heading level={1}>{channel.name}</Heading>{error&&<Banner status="error" title={error}/>}<form onSubmit={create}><Stack gap={3}><TextInput label="Title" value={title} onChange={setTitle} required/><TextArea label="Note" value={content} onChange={setContent}/><Button label="Add note" type="submit" variant="primary"/></Stack></form><Stack gap={3}>{loading&&<Spinner label="Loading notes…" size="lg"/>}{notes.map(note=><Card key={note.id} padding={3}><Stack gap={2}><Heading level={3}>{note.title}</Heading><Text>{note.content}</Text></Stack></Card>)}</Stack></Stack>;
}
