#!/usr/bin/env python3
"""Convert bazi notes (Markdown) into Confluence storage-format pages.

Usage: build.py <books_dir> <notes_dir> <out_dir>
  books_dir = repo docs/books
  notes_dir = /mnt/project-files/notes (lesson-summaries, chart-reviews, yongshen)
Writes <out_dir>/pages/NNNN.xml and <out_dir>/manifest.json.
"""
import json, os, re, sys, shutil, posixpath, unicodedata
from html import escape
import markdown
from lxml import etree, html as lhtml

BOOKS, NOTES, OUT = map(os.path.abspath, sys.argv[1:4])
SPACE_KEY = 'Bazi'
SPLIT_BYTES = 250_000  # split a source file by H1 when larger than this

pages = []          # {key,title,parent,file?,body?,attachments?}
by_path = {}        # abs source path (or dir path) -> page key
titles = set()


def add(key, title, parent, src=None, body=None, attachments=None, labels=(), aliases=(), leaf=True):
    assert title not in titles, title
    titles.add(title)
    pages.append(dict(key=key, title=title, parent=parent, src=src, body=body,
                      attachments=attachments or [], labels=list(labels), aliases=list(aliases), leaf=leaf))
    if src:
        by_path[os.path.abspath(src)] = key
    return key


CHILDREN = '<ac:structured-macro ac:name="children"><ac:parameter ac:name="all">true</ac:parameter></ac:structured-macro>'


def folder(key, title, parent, path=None, readme=None, intro=None, labels=()):
    k = add(key, title, parent, src=readme, labels=labels, leaf=False,
            body=None if readme else (f'<p>{escape(intro)}</p>' if intro else '') + CHILDREN)
    if path:
        by_path[os.path.abspath(path)] = k
    return k


def md_files(d):
    return sorted(f for f in os.listdir(d) if f.endswith('.md') and f != 'README.md')


# ---------- tree ----------
ROOT = 'ROOT'
B = folder('books', '書籍筆記', ROOT, BOOKS, os.path.join(BOOKS, 'README.md'), labels=['書籍筆記'])

AUTHORS = [
    ('河洛子', ['量化五行力', '八字真理', '滴天髓命例研究']),
    ('陸致極', ['基礎教程', '動態分析教程', '進階教程', '陸致極-強弱規則']),
    ('吳俊民', ['命理新論']),
    ('潘樂德', ['十神洩天機']),
    ('蘇民峰', ['蘇民峰-八字論命']),
    ('林信銘', ['妙算人生八字']),
    ('錢思吾', ['輕鬆學算命']),
    ('覺慧居士', ['八字心訣']),
    ('鄭偉雄', ['子平正源-轉盤論法']),
]
BOOK_TITLE = {
    '量化五行力': '河洛子《八字算命之量化五行力》', '八字真理': '河洛子《算命方程式@八字真理》',
    '滴天髓命例研究': '河洛子《滴天髓命例研究》', '基礎教程': '陸致極《八字命理學基礎教程》',
    '動態分析教程': '陸致極《八字命理學動態分析教程》', '進階教程': '陸致極《八字命理學進階教程》',
    '陸致極-強弱規則': '陸致極 日主強弱規則彙整', '命理新論': '吳俊民《命理新論》',
    '十神洩天機': '潘樂德《十神洩天機》', '蘇民峰-八字論命': '蘇民峰《玄學錦囊・八字論命》',
    '妙算人生八字': '林信銘《妙算人生八字》', '輕鬆學算命': '錢思吾《輕鬆學算命》',
    '八字心訣': '覺慧居士《八字心訣》', '子平正源-轉盤論法': '鄭偉雄《子平正源——轉盤論法》',
}
BOOK_LABEL = {'陸致極-強弱規則': '陸致極強弱規則', '蘇民峰-八字論命': '八字論命', '子平正源-轉盤論法': '子平正源'}
# prefix for generic file names, so titles stay unique within the space
PREFIX = {'滴天髓命例研究': '滴天髓命例研究 ', '蘇民峰-八字論命': '蘇民峰八字論命 ', '中級二課本': '中級二課本 ',
          '妙算人生八字': '妙算人生八字 ', '輕鬆學算命': '輕鬆學算命 ', '八字心訣': '八字心訣 ', '子平正源-轉盤論法': '子平正源 '}


def file_title(book, fname):
    t = fname[:-3]
    p = PREFIX.get(book, '')
    return t if t.startswith(p.strip()) else p + t


def add_file(key, title, parent, path, labels=()):
    """Add one Markdown file; split into child pages by H1 when it is large."""
    if os.path.getsize(path) <= SPLIT_BYTES:
        return add(key, title, parent, src=path, labels=labels)
    text = open(path, encoding='utf-8').read()
    parts = re.split(r'(?m)^(?=# )', text)
    head = [p for p in parts if p.strip() and not p.startswith('# ')]
    k = add(key, title, parent, src=path, body=None, labels=labels, leaf=False)
    pages[-1]['md'] = (head[0] if head else '') + '\n\n' + '<!--CHILDREN-->'
    for i, p in enumerate(p for p in parts if p.startswith('# ')):
        h = p.split('\n', 1)[0][2:].strip()
        add(f'{key}#{i}', f'{title} {h}', k, labels=labels)
        pages[-1]['md'] = p
        pages[-1]['srcdir'] = os.path.dirname(path)
    return k


for author, books in AUTHORS:
    a = folder(f'author:{author}', f'{author} 書籍筆記', B, intro=f'{author} 著作的整理筆記。', labels=['書籍筆記', author])
    for book in books:
        d = os.path.join(BOOKS, book)
        readme = os.path.join(d, 'README.md')
        bl = ['書籍筆記', author, BOOK_LABEL.get(book, book)]
        bk = folder(f'book:{book}', BOOK_TITLE[book], a, d, readme if os.path.exists(readme) else None, labels=bl)
        if book == '命理新論':
            for vol in ['上冊', '中冊', '下冊']:
                vd = os.path.join(d, vol)
                vk = folder(f'book:命理新論:{vol}', f'命理新論{vol}', bk, vd, labels=bl)
                for f in md_files(vd):
                    add_file(f'f:{vd}/{f}', file_title(book, f), vk, os.path.join(vd, f), labels=bl)
            continue
        for f in md_files(d):
            add_file(f'f:{d}/{f}', file_title(book, f), bk, os.path.join(d, f), labels=bl)

_cmp = os.path.join(BOOKS, '四本書特色比較.md')
if os.path.exists(_cmp):
    add('books:compare4', '四本書特色比較（妙算・輕鬆・心訣・子平正源）', B, _cmp, labels=['書籍筆記'])

LL = ['上課筆記', '許羽賢']
L = folder('lessons', '上課筆記（許羽賢老師）', ROOT,
           intro='許羽賢老師子平八字課程（新北，每週一堂）的整理：課程總結、每週筆記、講義課本與專題。', labels=LL)
LS = os.path.join(NOTES, 'lesson-summaries')
add('lesson:summary', '課程總結', L, os.path.join(LS, '課程總結-20260701-20260923.md'),
    attachments=['2026-10-21-whiteboard-69female.png'], labels=LL)
pages[-1]['attachdir'] = LS
W = folder('lessons:weekly', '每週上課筆記', L, labels=LL)
LESSONS = []
for f in sorted(f for f in os.listdir(LS) if re.match(r'\d{4}-\d\d-\d\d\.md$', f)):
    first = open(os.path.join(LS, f), encoding='utf-8').readline()
    sub = first.split('｜', 1)[1].strip() if '｜' in first else ''
    short = re.split(r'[：，]', sub)[0].strip()
    t = f'上課筆記 {f[:-3]} {short}'.strip()
    add(f'lesson:{f}', t, W, os.path.join(LS, f), labels=LL, aliases=[f'上課筆記 {f[:-3]}'],
        attachments=['2026-10-21-whiteboard-69female.png'] if f == '2026-10-21.md' else None)
    pages[-1]['attachdir'] = LS
    LESSONS.append((f[:-3], sub, t))
d = os.path.join(BOOKS, '中級二課本')
CL = LL + ['中級二課本']
ck = folder('book:中級二課本', '中級二課本（講義第 23–26 章）', L, d, os.path.join(d, 'README.md'), labels=CL)
for f in md_files(d):
    add(f'f:{d}/{f}', file_title('中級二課本', f), ck, os.path.join(d, f), labels=CL)
add('topic:yongshen', '如何取用神', L, os.path.join(NOTES, 'yongshen', '如何取用神.md'), labels=LL)

C = folder('charts', '命例分析', ROOT, intro='實際命盤的解讀紀錄。', labels=['命例分析'])
CR = os.path.join(NOTES, 'chart-reviews')
for f in sorted(os.listdir(CR)):
    if f.endswith('.md'):
        add(f'chart:{f}', f'命例 {f[:-3]}', C, os.path.join(CR, f), labels=['命例分析'])


sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from topics import TOPICS, THRESHOLD
T = folder('topics', '主題索引', ROOT, labels=['主題索引'])
for name, desc, _ in TOPICS:
    add(f'topic-index:{name}', f'主題：{name}', T, body='', labels=['主題索引'], leaf=False)
add('HOME', '八字命理', None, body='')

# ---------- conversion ----------
def gh_slug(s):
    s = s.strip().lower()
    s = ''.join(c for c in s if c in ' -_' or unicodedata.category(c)[0] in 'LN')
    return s.replace(' ', '-')


def page_link(title, text, anchor=None):
    a = f' ac:anchor="{escape(anchor)}"' if anchor else ''
    return (f'<ac:link{a}><ri:page ri:content-title="{escape(title)}"/>'
            f'<ac:plain-text-link-body><![CDATA[{text}]]></ac:plain-text-link-body></ac:link>')


title_of = {p['key']: p['title'] for p in pages}
MD = markdown.Markdown(extensions=['tables', 'fenced_code', 'sane_lists'])


def convert(md_text, srcdir):
    MD.reset()
    md_text = md_text.replace('<!--CHILDREN-->', '<p>@@CHILDREN@@</p>')
    out = MD.convert(md_text)
    root = lhtml.fragment_fromstring(out, create_parent='div')
    anchors = set()
    links = []
    for a in root.iter('a'):
        href = a.get('href', '')
        if href.startswith('#'):
            anchors.add(href[1:])
        links.append(a)
    # heading anchors that something links to
    for h in root.iter('h1', 'h2', 'h3', 'h4', 'h5', 'h6'):
        slug = gh_slug(h.text_content())
        if slug in anchors:
            m = etree.SubElement(h, 'ANCHOR')
            m.text = slug
    repl = {}
    for i, a in enumerate(links):
        href = a.get('href', '')
        text = a.text_content()
        if re.match(r'^[a-z]+:', href):
            continue
        if href.startswith('#'):
            repl[i] = f'<ac:link ac:anchor="{escape(href[1:])}"><ac:plain-text-link-body><![CDATA[{text}]]></ac:plain-text-link-body></ac:link>'
        else:
            path, _, frag = href.partition('#')
            from urllib.parse import unquote
            tgt = os.path.normpath(os.path.join(srcdir, unquote(path)))
            key = by_path.get(tgt) or by_path.get(tgt.rstrip('/'))
            if key:
                repl[i] = page_link(title_of[key], text)
            elif re.search(r'\.(png|jpe?g|gif)$', path, re.I):
                repl[i] = f'<ac:image><ri:attachment ri:filename="{escape(os.path.basename(unquote(path)))}"/></ac:image>'
            else:
                repl[i] = escape(text)
        tail = a.tail
        a.clear()
        a.tag = 'REPL'
        a.text = str(i)
        a.tail = tail
    for img in list(root.iter('img')):
        src = img.get('src', '')
        img.tag = 'IMG'
        img.attrib.clear()
        img.text = os.path.basename(src)
    for pre in list(root.iter('pre')):
        code = pre.find('code')
        lang = (code.get('class', '') if code is not None else '').replace('language-', '')
        txt = pre.text_content()
        pre.clear()
        pre.tag = 'CODE'
        pre.set('lang', lang)
        pre.text = txt
    for det in list(root.iter('details')):
        det.tag = 'EXPAND'
        s = det.find('summary')
        if s is not None:
            det.set('title', s.text_content().strip())
            det.remove(s)
    xml = etree.tostring(root, encoding='unicode', method='xml')
    xml = xml[len('<div>'):-len('</div>')] if xml.startswith('<div>') else xml
    xml = re.sub(r'<REPL>(\d+)</REPL>', lambda m: repl[int(m.group(1))], xml)
    xml = re.sub(r'<ANCHOR>(.*?)</ANCHOR>', lambda m: f'<ac:structured-macro ac:name="anchor"><ac:parameter ac:name="">{m.group(1)}</ac:parameter></ac:structured-macro>', xml)
    xml = re.sub(r'<IMG>(.*?)</IMG>', lambda m: f'<ac:image><ri:attachment ri:filename="{m.group(1)}"/></ac:image>', xml)

    def code_macro(m):
        from html import unescape
        lang, body = m.group(1), unescape(m.group(2)).replace(']]>', ']]]]><![CDATA[>')
        lp = f'<ac:parameter ac:name="language">{lang}</ac:parameter>' if lang else ''
        return f'<ac:structured-macro ac:name="code">{lp}<ac:plain-text-body><![CDATA[{body}]]></ac:plain-text-body></ac:structured-macro>'
    xml = re.sub(r'<CODE lang="([^"]*)">(.*?)</CODE>', code_macro, xml, flags=re.S)
    xml = re.sub(r'<CODE lang="([^"]*)"/>', '', xml)
    xml = re.sub(r'<EXPAND(?: title="([^"]*)")?>(.*?)</EXPAND>',
                 lambda m: f'<ac:structured-macro ac:name="expand"><ac:parameter ac:name="title">{m.group(1) or ""}</ac:parameter><ac:rich-text-body>{m.group(2)}</ac:rich-text-body></ac:structured-macro>',
                 xml, flags=re.S)
    xml = xml.replace('<p>@@CHILDREN@@</p>', CHILDREN)
    return xml


def check_wellformed(xml):
    wrapped = ('<r xmlns:ac="http://atlassian.com/content" xmlns:ri="http://atlassian.com/resource/identifier">'
               + xml + '</r>')
    etree.fromstring(wrapped.encode())


shutil.rmtree(OUT, ignore_errors=True)
os.makedirs(os.path.join(OUT, 'pages'))
os.makedirs(os.path.join(OUT, 'attachments'))
P = {p['key']: p for p in pages}


def topic_scores(xml):
    t = re.sub(r'<table>.*?</table>', '', xml, flags=re.S)
    t = re.sub(r'<[^>]+>', '', t)
    n = max(len(t), 3000)
    sc = [(sum(t.count(k) for k in kws) / n * 10000, name) for name, _, kws in TOPICS]
    return sorted(sc, reverse=True)


def toc_macro(xml):
    h2 = len(re.findall(r'<h2[ >]', xml))
    h3 = len(re.findall(r'<h3[ >]', xml))
    if h2 + h3 < 3 or h2 > 40:
        return ''
    lvl = 3 if h2 + h3 <= 60 else 2
    return ('<ac:structured-macro ac:name="expand"><ac:parameter ac:name="title">本頁目錄</ac:parameter><ac:rich-text-body>'
            f'<ac:structured-macro ac:name="toc"><ac:parameter ac:name="maxLevel">{lvl}</ac:parameter>'
            '<ac:parameter ac:name="minLevel">2</ac:parameter></ac:structured-macro></ac:rich-text-body></ac:structured-macro>')


def link(key, text=None):
    return page_link(P[key]['title'], text or P[key]['title'])


# 1. convert content pages, assign topic labels
for p in pages:
    if p.get('md') is not None:
        p['xml'] = convert(p['md'], p.get('srcdir') or os.path.dirname(p['src']))
    elif p['src']:
        md = open(p['src'], encoding='utf-8').read()
        md = md.replace('（函授講義，上冊已整理；中、下冊待補）', '（函授講義，上、中、下冊）')
        p['xml'] = convert(md, os.path.dirname(p['src']))
    else:
        p['xml'] = p['body']
    if p['leaf'] and p['key'] != 'HOME' and not p['key'].startswith('topic-index:'):
        p['topics'] = [nm for sc, nm in topic_scores(p['xml'])[:4] if sc >= THRESHOLD.get(nm, 20)]
        p['labels'] += p['topics']

# 2. topic pages: static lists grouped by category
CATS = [('書籍筆記', '書籍筆記'), ('上課筆記', '上課筆記'), ('命例分析', '命例分析')]
for name, desc, kws in TOPICS:
    tp = P[f'topic-index:{name}']
    out = [f'<p>{escape(desc)}。</p>',
           f'<p>關鍵詞：{escape("、".join(kws))}。下面列出以這個主題為重點的頁面（依內容自動判定，也可以搜尋 <code>label:{escape(name)}</code>）。</p>']
    for cat, lab in CATS:
        items = [q for q in pages if name in q.get('topics', []) and lab in q['labels']]
        if items:
            out.append(f'<h2>{cat}（{len(items)}）</h2><ul>' + ''.join(
                f'<li>{link(q["key"])} <span style="color: rgb(122,134,154);">— {escape(P[q["parent"]]["title"])}</span></li>' for q in items) + '</ul>')
    tp['xml'] = ''.join(out)
    tp['count'] = sum(1 for q in pages if name in q.get('topics', []))
P['topics']['xml'] = ('<p>依主題找筆記。每個主題頁列出書籍、上課、命例裡以該主題為重點的頁面。</p>'
    '<table><tbody><tr><th>主題</th><th>內容</th><th>頁數</th></tr>' + ''.join(
    f'<tr><td>{link(f"topic-index:{n}", n)}</td><td>{escape(d)}</td><td>{P[f"topic-index:{n}"]["count"]}</td></tr>'
    for n, d, _ in TOPICS) + '</tbody></table>')

# 3. weekly lessons index
P['lessons:weekly']['xml'] = ('<p>每堂課的錄音整理（語音轉文字，四柱標「待核對」者尚未對照白板或講義）。</p>'
    '<table><tbody><tr><th>日期</th><th>主題</th><th>重點標籤</th></tr>' + ''.join(
    f'<tr><td>{link(f"lesson:{d}.md", d)}</td><td>{escape(sub)}</td><td>{escape("、".join(P[f"lesson:{d}.md"]["topics"]))}</td></tr>'
    for d, sub, t in LESSONS) + '</tbody></table>')

# 4. home page
def ul(keys):
    return '<ul>' + ''.join(f'<li>{link(k)}</li>' for k in keys) + '</ul>'
books_html = ''
for author, bks in AUTHORS:
    books_html += f'<p><strong>{author}</strong></p>' + ul([f'book:{b}' for b in bks])
recent = [f'lesson:{d}.md' for d, _, _ in LESSONS[-4:]][::-1]
P['HOME']['xml'] = (
    '<ac:structured-macro ac:name="anchor"><ac:parameter ac:name="">bazi-home</ac:parameter></ac:structured-macro>'
    '<ac:structured-macro ac:name="info"><ac:rich-text-body><p>子平八字學習筆記：書籍整理、許羽賢老師上課筆記、實際命例分析。'
    '命盤一律照 App 排法（時、日、月、年），天干地支後的小字是五行。</p></ac:rich-text-body></ac:structured-macro>'
    '<h2>搜尋</h2><ac:structured-macro ac:name="livesearch"><ac:parameter ac:name="spaceKey"><ri:space ri:space-key="' + SPACE_KEY + '"/></ac:parameter>'
    '<ac:parameter ac:name="placeholder">搜尋本空間，例如：合而不化、從財、調候</ac:parameter></ac:structured-macro>'
    '<p>小技巧：搜尋 <code>label:用神</code> 只找有該標籤的頁面；每頁上方的「本頁目錄」可以展開跳到段落。</p>'
    '<ac:layout><ac:layout-section ac:type="three_equal">'
    '<ac:layout-cell><h2>依主題找</h2><ul>' + ''.join(
        f'<li>{link(f"topic-index:{n}", n)}（{P[f"topic-index:{n}"]["count"]}）</li>' for n, _, _ in TOPICS) + '</ul></ac:layout-cell>'
    '<ac:layout-cell><h2>上課筆記</h2>' + ul(['lesson:summary', 'lessons:weekly', 'book:中級二課本', 'topic:yongshen']) +
    '<p><strong>最近幾堂</strong></p>' + ul(recent) +
    '<h2>命例分析</h2>' + ul([q['key'] for q in pages if q['parent'] == 'charts']) + '</ac:layout-cell>'
    '<ac:layout-cell><h2>書籍筆記</h2>' + books_html + '</ac:layout-cell>'
    '</ac:layout-section><ac:layout-section ac:type="single"><ac:layout-cell>'
    '<h2>最近更新</h2><ac:structured-macro ac:name="recently-updated"><ac:parameter ac:name="spaces"><ri:space ri:space-key="' + SPACE_KEY + '"/></ac:parameter>'
    '<ac:parameter ac:name="max">10</ac:parameter></ac:structured-macro>'
    '<h2>全部頁面</h2><ac:structured-macro ac:name="pagetree"><ac:parameter ac:name="root"><ac:link><ri:page ri:content-title="八字命理"/></ac:link></ac:parameter>'
    '<ac:parameter ac:name="searchBox">true</ac:parameter><ac:parameter ac:name="expandCollapseAll">true</ac:parameter></ac:structured-macro>'
    '</ac:layout-cell></ac:layout-section></ac:layout>')

# 5. toc + prev/next navigation on leaf pages
for p in pages:
    if p['key'] == 'HOME':
        continue
    if p['leaf'] and not p['key'].startswith('topic-index:'):
        sib = [q for q in pages if q['parent'] == p['parent'] and q['leaf']]
        i = sib.index(p)
        nav = []
        if i > 0: nav.append('← 上一篇：' + link(sib[i - 1]['key']))
        nav.append('↑ ' + link(p['parent']))
        if i + 1 < len(sib): nav.append('下一篇：' + link(sib[i + 1]['key']) + ' →')
        tags = '、'.join(link(f'topic-index:{t}', t) for t in p.get('topics', []))
        bar = '<p>' + '　｜　'.join(nav) + '</p>'
        p['xml'] = (toc_macro(p['xml']) + p['xml'] + '<hr/>' + bar +
                    (f'<p>相關主題：{tags}</p>' if tags else ''))

manifest = []
for n, p in enumerate(pages):
    xml = p['xml']
    check_wellformed(xml)
    fn = f'pages/{n:04d}.xml'
    open(os.path.join(OUT, fn), 'w', encoding='utf-8').write(xml)
    for a in p['attachments']:
        shutil.copy(os.path.join(p['attachdir'], a), os.path.join(OUT, 'attachments', a))
    manifest.append(dict(key=p['key'], title=p['title'], parent=p['parent'], file=fn,
                         attachments=p['attachments'], labels=sorted(set(p['labels'])),
                         aliases=p['aliases'], bytes=len(xml.encode())))
json.dump(manifest, open(os.path.join(OUT, 'manifest.json'), 'w', encoding='utf-8'),
          ensure_ascii=False, indent=1)
print(len(manifest), 'pages; largest', max(m['bytes'] for m in manifest))
