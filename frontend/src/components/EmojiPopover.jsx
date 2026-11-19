import {useEffect,useState,useRef} from 'react';
import {createPortal} from 'react-dom';
import EmojiPicker from 'emoji-picker-react';
export function EmojiPopover({anchor,onPick,onClose,theme}){
 const [closing,setClosing]=useState(false),node=useRef(null),timer=useRef(null);
 function close(){if(closing)return;setClosing(true);timer.current=setTimeout(onClose,130);}
 useEffect(()=>{const listener=e=>{if(e.type==='keydown'&&e.key!=='Escape')return;if(node.current?.contains(e.target))return;close();};document.addEventListener('pointerdown',listener);document.addEventListener('keydown',listener);return()=>{document.removeEventListener('pointerdown',listener);document.removeEventListener('keydown',listener);clearTimeout(timer.current);};},[]);
 const w=Math.min(320,window.innerWidth-16),h=Math.min(390,window.innerHeight-16),x=Math.max(8,Math.min(window.innerWidth-w-8,anchor.left)),y=anchor.top>=h+8?anchor.top-h-8:Math.min(window.innerHeight-h-8,anchor.bottom+8);
 return createPortal(<div ref={node} className={`anchored-emoji ${closing?'closing':''}`} style={{left:x,top:Math.max(8,y)}}><button type="button" className="picker-close" aria-label="Close emoji picker" onClick={close}>×</button><EmojiPicker width={w} height={h} emojiStyle="native" theme={theme} onEmojiClick={e=>{onPick(e.emoji);close();}} searchPlaceholder="Search emoji" lazyLoadEmojis previewConfig={{showPreview:false}}/></div>,document.body);
}
