#!/usr/bin/env node
/* Upgrade the existing installation in place; do not touch .env, databases or uploads. */
const fs=require('node:fs');
const path=require('node:path');
const net=require('node:net');
const http=require('node:http');
const readline=require('node:readline/promises');
const {spawn,spawnSync}=require('node:child_process');
const source=__dirname;
const sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms));
function fail(text){console.error('\n'+text);process.exitCode=1;}
function run(program,args,cwd){const result=spawnSync(program,args,{cwd,stdio:'inherit',shell:process.platform==='win32'&&program==='npm'});if(result.error)throw new Error(`Could not run ${program}: ${result.error.message}`);if(result.status!==0)throw new Error(`${program} failed (exit ${result.status}). No data or configuration was intentionally changed.`);}
function inUse(port,host){return new Promise(resolve=>{const socket=net.createConnection({port,host});socket.setTimeout(1200);socket.once('connect',()=>{socket.destroy();resolve(true)});socket.once('error',()=>resolve(false));socket.once('timeout',()=>{socket.destroy();resolve(false)});});}
function version(port){return new Promise(resolve=>{const r=http.get({hostname:'127.0.0.1',port,path:'/api/version',timeout:2000},res=>{let b='';res.on('data',x=>b+=x);res.on('end',()=>{try{resolve(JSON.parse(b));}catch{resolve(null);}})});r.on('error',()=>resolve(null));r.on('timeout',()=>r.destroy());});}
function copyDirectory(from,to){fs.mkdirSync(to,{recursive:true});fs.cpSync(from,to,{recursive:true,force:true});}
function copyFile(from,to){fs.mkdirSync(path.dirname(to),{recursive:true});fs.copyFileSync(from,to);}
function stopChild(child){if(!child||child.exitCode!==null)return;try{if(process.platform==='win32')spawnSync('taskkill',['/pid',String(child.pid),'/t','/f'],{stdio:'ignore'});else child.kill('SIGTERM');}catch{}}
async function main(){
 console.log('Boron in-place upgrade — keeps your existing .env, database and uploads.');
 let target=process.argv[2];
 if(!target){const rl=readline.createInterface({input:process.stdin,output:process.stdout});target=await rl.question('Paste the path to your EXISTING Boron project folder: ');rl.close();}
 target=path.resolve((target||'').trim().replace(/^"|"$/g,''));
 if(path.basename(target).toLowerCase()==='backend'&&fs.existsSync(path.join(target,'Cargo.toml'))&&fs.existsSync(path.join(path.dirname(target),'frontend','package.json')))target=path.dirname(target);
 const back=path.join(target,'backend'),front=path.join(target,'frontend');
 if(!fs.existsSync(path.join(back,'Cargo.toml'))||!fs.existsSync(path.join(front,'package.json')))throw new Error('That folder must contain your existing backend and frontend folders. Nothing was changed.');
 if(!fs.existsSync(path.join(back,'.env'))&&!process.env.DATABASE_URL)throw new Error('No backend/.env or DATABASE_URL was found. Point this tool at your existing installation with its configuration, or set DATABASE_URL first. Nothing was changed.');
 const code=fs.readFileSync(path.join(source,'backend','src','main.rs'),'utf8');const revision=Number(code.match(/"api_revision"\s*:\s*(\d+)/)?.[1]||0);
 if(revision<7||!fs.readFileSync(path.join(source,'backend','src','routes','channels.rs'),'utf8').includes('/:id/pins'))throw new Error('This downloaded update does not contain the revision 7 pins route. Nothing was changed.');
 for(const port of [3001,57575])if((await inUse(port,'127.0.0.1'))||(await inUse(port,'::1')))throw new Error(`Port ${port} is in use. Stop the old Boron backend and frontend (Ctrl+C in their terminals), then run this again. No process was stopped and no files were changed. On Windows, run netstat -ano | findstr :${port} to identify it; do not stop unrelated services.`);
 console.log(`Updating source in: ${target}\nSource revision: ${revision}\nExisting configuration, database and uploads will be kept.`);
 if(path.resolve(source)!==target){
  for(const dir of ['src','migrations'])copyDirectory(path.join(source,'backend',dir),path.join(back,dir));
  for(const file of ['Cargo.toml','Cargo.lock'])copyFile(path.join(source,'backend',file),path.join(back,file));
  copyDirectory(path.join(source,'frontend','src'),path.join(front,'src'));
  for(const file of ['package.json','package-lock.json','index.html','vite.config.js'])copyFile(path.join(source,'frontend',file),path.join(front,file));
 }
 console.log('\nCleaning only the Boron compiled binary and rebuilding the backend…');
 run('cargo',['clean','-p','boron-backend'],back);
 run('cargo',['build','--locked'],back);
 console.log('\nInstalling the frontend dependencies…');run('npm',['ci'],front);
 console.log('\nStarting the updated backend and frontend. Keep this window open; press Ctrl+C to stop both.');
 const backend=spawn('cargo',['run','--locked'],{cwd:back,stdio:'inherit'});
 const frontend=spawn('npm',['run','dev','--','--host','127.0.0.1','--port','57575','--strictPort'],{cwd:front,stdio:'inherit',shell:process.platform==='win32'});
 let closing=false;
 const cleanup=()=>{if(closing)return;closing=true;stopChild(frontend);stopChild(backend);};
 process.once('SIGINT',()=>{cleanup();process.exit(0);});process.once('SIGTERM',()=>{cleanup();process.exit(0);});
 for(const [label,child] of [['Backend',backend],['Frontend',frontend]]){
  child.on('error',e=>{console.error(`${label} failed to start: ${e.message}`);cleanup();process.exitCode=1;});
  child.on('exit',code=>{if(!closing){console.error(`${label} exited (${code}). Stopping the other service.`);cleanup();process.exitCode=1;}});
 }
 let verified=false;
 for(let i=0;i<60&&!closing;i++){const [direct,proxied]=await Promise.all([version(3001),version(57575)]);if(direct?.service==='boron-backend'&&proxied?.service==='boron-backend'&&direct.api_revision>=7&&proxied.api_revision>=7){console.log(`\nVerified: backend revision ${direct.api_revision}; frontend proxy revision ${proxied.api_revision}. Open http://localhost:57575 and refresh.`);verified=true;break;}await sleep(1000);}
 if(!verified&&!closing)console.error('\nCould not verify both URLs within 60 seconds. Check the output above; services remain running until you press Ctrl+C.');
}
main().catch(e=>fail(e.message));
