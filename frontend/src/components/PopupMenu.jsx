import {useEffect} from 'react';
import {createPortal} from 'react-dom';
export function PopupMenu({menu,onClose}){
 useEffect(()=>{if(!menu)return;const close=e=>{if(e.type==='keydown'&&e.key!=='Escape')return;if(e.target?.closest?.('.popup-menu'))return;onClose();};document.addEventListener('pointerdown',close);document.addEventListener('keydown',close);window.addEventListener('resize',close);return()=>{document.removeEventListener('pointerdown',close);document.removeEventListener('keydown',close);window.removeEventListener('resize',close);};},[menu,onClose]);
 if(!menu)return null;return createPortal(<div className="popup-menu" role="menu" style={{left:Math.min(menu.x,window.innerWidth-210),top:Math.min(menu.y,window.innerHeight-menu.items.length*40-12)}}>{menu.items.map((item,i)=><button role="menuitem" key={i} type="button" className={item.destructive?'destructive':''} onClick={()=>{onClose();item.onClick();}}>{item.label}</button>)}</div>,document.body);
}
