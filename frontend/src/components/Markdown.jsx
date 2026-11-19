import ReactMarkdown, {defaultUrlTransform} from 'react-markdown';
import remarkGfm from 'remark-gfm';
import remarkBreaks from 'remark-breaks';
import rehypeHighlight from 'rehype-highlight';
export function Markdown({children}) {
 return <article className="markdown"><ReactMarkdown remarkPlugins={[remarkGfm,remarkBreaks]} rehypePlugins={[rehypeHighlight]} urlTransform={url=>url.startsWith('boron-user:')?url:defaultUrlTransform(url)} components={{a:({href,children,...props})=>href?.startsWith('boron-user:')?<strong className="mention">{children}</strong>:<a href={href} {...props} rel="noopener noreferrer">{children}</a>,img:({src,alt})=><img src={src} alt={alt} loading="lazy"/>}}>{children||''}</ReactMarkdown></article>;
}
