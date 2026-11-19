import { api } from './api';
export const MAX_FILE = 800 * 1024 * 1024;
export async function uploadFile(url, file) {
 if (file.size > MAX_FILE) throw new Error('Files must be 800 MiB or smaller.');
 const data = new FormData(); data.append('file', file);
 const response = await api.post(url, data, { headers: { 'Content-Type': undefined } });
 return response.data;
}
export async function fileUrl(id) {
 const response=await api.get(`/api/files/${id}/ticket`);
 return new URL(response.data.url,api.defaults.baseURL||window.location.origin).href;
}
export async function downloadFile(id,filename) {
 const url=await fileUrl(id); const link=document.createElement('a');link.href=url+"?download=1";link.download=filename;link.referrerPolicy='no-referrer';link.click();
}
