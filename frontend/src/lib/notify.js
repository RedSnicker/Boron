let sound;
export function enableSound(){try{sound ||= new (window.AudioContext||window.webkitAudioContext)();sound.resume();}catch{}}
export function ping(){try{if(!sound)return;const now=sound.currentTime;for(const [freq,delay] of [[880,0],[1320,.09]]){const o=sound.createOscillator(),g=sound.createGain();o.type='sine';o.frequency.value=freq;g.gain.setValueAtTime(.0001,now+delay);g.gain.exponentialRampToValueAtTime(.11,now+delay+.015);g.gain.exponentialRampToValueAtTime(.0001,now+delay+.18);o.connect(g);g.connect(sound.destination);o.start(now+delay);o.stop(now+delay+.19);}}catch{}}
export async function requestNotifications(){enableSound();if(!('Notification' in window))return 'unsupported';if(Notification.permission==='default')return Notification.requestPermission();return Notification.permission;}
export function notify(title,body,onClick){ping();if('Notification' in window&&Notification.permission==='granted'){try{const n=new Notification(title,{body,tag:`boron-${Date.now()}`});n.onclick=()=>{window.focus();onClick?.();n.close();};}catch{}}}
// A brief, soft paper burst with an ascending chime for a completed issue.
export function celebrateSound(){try{
 if(!sound)return;
 const now=sound.currentTime,duration=.22,buffer=sound.createBuffer(1,Math.ceil(sound.sampleRate*duration),sound.sampleRate),samples=buffer.getChannelData(0);
 for(let i=0;i<samples.length;i++)samples[i]=(Math.random()*2-1)*Math.pow(1-i/samples.length,3);
 const burst=sound.createBufferSource(),gain=sound.createGain();burst.buffer=buffer;gain.gain.setValueAtTime(.075,now);gain.gain.exponentialRampToValueAtTime(.0001,now+duration);burst.connect(gain);gain.connect(sound.destination);burst.start(now);burst.stop(now+duration);
 for(const [freq,delay] of [[660,.02],[880,.10],[1175,.18]]){const o=sound.createOscillator(),g=sound.createGain();o.type='sine';o.frequency.value=freq;g.gain.setValueAtTime(.0001,now+delay);g.gain.exponentialRampToValueAtTime(.055,now+delay+.012);g.gain.exponentialRampToValueAtTime(.0001,now+delay+.20);o.connect(g);g.connect(sound.destination);o.start(now+delay);o.stop(now+delay+.21);}
 }catch{}}
