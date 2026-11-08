#!/usr/bin/env python3
"""Merge all bazi book notes + lesson notes into one EPUB 3 with a nested TOC.

usage: build.py <repo_root> <lesson_dir> <out.epub>
"""
import csv, datetime, html, os, re, subprocess, sys, uuid, zipfile
from pathlib import Path
from lxml import etree, html as lhtml

REPO = Path(sys.argv[1]).resolve()
LESSON = Path(sys.argv[2]).resolve()
OUT = Path(sys.argv[3]).resolve()
BOOKS = REPO / 'docs' / 'books'
TODAY = datetime.date.today().isoformat()


class Node:
    def __init__(self, title, kind, src=None, md=None, children=None, short=None):
        self.title, self.kind, self.src, self.md = title, kind, src, md
        self.short = short or title
        self.children = children or []
        self.file = None        # output xhtml name
        self.heads = []         # [(level, text, id)]
        self.body = None        # lxml element
        self.parent = None


def md_title(text):
    m = re.search(r'(?m)^#\s+(.+)$', text)
    return clean(m.group(1)) if m else None


def clean(s):
    s = re.sub(r'<[^>]+>', '', s)
    s = re.sub(r'[*_`]', '', s)
    s = re.sub(r'\[([^\]]*)\]\([^)]*\)', r'\1', s)
    return s.strip()


def chapter(path, title=None, md=None, short=None):
    path = Path(path)
    text = md if md is not None else path.read_text(encoding='utf-8')
    # normalise headings so every file has exactly one H1 at top
    lv = [len(m.group(1)) for m in re.finditer(r'(?m)^(#{1,6})\s', text)]
    h1 = lv.count(1)
    if h1 == 0 and lv:
        shift = min(lv) - 1
        text = re.sub(r'(?m)^(#{1,6})(\s)', lambda m: '#' * max(1, len(m.group(1)) - shift) + m.group(2), text)
    elif h1 > 1:
        t = title or md_title(text)
        text = re.sub(r'(?m)^(#{1,5})(\s)', lambda m: '#' + m.group(1) + m.group(2), text)
        text = f'# {t}\n\n' + text
    t = title or md_title(text) or path.stem
    if title:
        text = re.sub(r'(?m)^#\s+.+$', '# ' + title, text, count=1)
    return Node(t, 'chapter', src=path, md=text, short=short)


def book(dirname, title, short=None, order=None, exclude=()):
    d = BOOKS / dirname
    files = sorted(p for p in d.glob('*.md') if p.name != 'README.md' and p.name not in exclude)
    if order:
        files = [d / f for f in order]
    b = Node(title, 'book', src=d, short=short or title)
    b.readme = d / 'README.md' if (d / 'README.md').exists() else None
    b.children = [chapter(f) for f in files]
    b.tsv = d / '命例.tsv' if (d / '命例.tsv').exists() else None
    return b


# ---------------------------------------------------------------- structure
EDITOR_NOTE = f"""# 編輯說明

這本 EPUB 把專案裡**自己整理的筆記**合成一冊（不是原書內容），{TODAY} 產生。

## 怎麼找

- 閱讀器的「目錄」是巢狀的：**部 → 書 → 章（檔）→ 節**。展開到節就能直接跳。
- 書的第一頁是「章節索引」，列出每章與章內各節；每章開頭也有「本章目錄」（節與小節）。
- 最前面的「總索引」一頁列出所有書與章節，隨時可從目錄回到這頁。
- 有命例表（命例.tsv）的書，另有「命例索引」頁：依編號列出四柱，點標題跳到該命例。

## 收錄範圍與取捨

- **上課資料**：許羽賢老師課程 2026-07-01 ～ 10-28 共 17 堂（09-16 沒有逐字稿）的逐堂筆記；「課程總結」只收前面的總覽、核心觀念與學員命例速查，各堂內容不重複放。另收 repo 根目錄的「上課筆記整理」（07-01～09-02 主題式重編）。老師的講義《中級二課本》（第 23–26 章）也放在上課資料。
- **吳俊民《命理新論》**：上冊用 2026-11-07 的「按章節詳細筆記」（20 章），中冊用「按節詳細筆記」（17 節），下冊用「章節筆記」（9 章）。舊的上冊主題筆記（13 檔）與三冊的「逐頁筆記」內容已被章節筆記取代，**沒有收進來**（逐頁筆記另有單冊 EPUB）。
- 其他書：docs/books 各資料夾的所有筆記都收錄，順序依檔名（頁碼或章次）。
- 命例四柱一律是 App 排盤格式（時 日 月 年，五行小字）。標「⚠ 四柱待核對」的命例照原筆記保留。
- 筆記裡指向本書以外檔案的連結（App 原始碼、逐頁筆記等）只保留文字。
"""

lessons = sorted(p for p in LESSON.glob('2026-*.md'))
summary = (LESSON / '課程總結-20260701-20260923.md').read_text(encoding='utf-8')
overview = re.split(r'(?m)^## 20\d\d-', summary)[0]
overview = re.sub(r'\n---\s*\n\s*<a id=[^\n]*>\s*$', '\n', overview.rstrip()) + '\n'
overview = re.sub(r'(?m)^#\s+.+$', '# 課程總覽（2026-07-01 ～ 10-28）', overview, count=1)
overview += '\n> 10-28 的內容見該堂筆記（本書已收錄）。\n'

course = Node('許羽賢老師 子平八字課程', 'book', short='許羽賢老師課程')
course.readme = None
course.tsv = None
course.children = [chapter(LESSON / '課程總覽.md', md=overview)] + [chapter(p) for p in lessons]
course.children.append(chapter(REPO / '上課筆記整理.md', title='上課筆記整理（07-01～09-02 主題式重編）'))

textbook = book('中級二課本', '中級二課本（許羽賢 講義 第 23–26 章）', short='中級二課本')

mlxl = Node('吳俊民《命理新論》', 'book', src=BOOKS / '命理新論', short='命理新論')
mlxl.readme = BOOKS / '命理新論' / 'README.md'
mlxl.tsv = None
vols = []
for vol, pat, label in [('上冊', '第??章-*.md', '上冊（初級，第 1–20 章）'),
                        ('中冊', '第??章第?節-*.md', '中冊（高級，第 21–23 章）'),
                        ('下冊', '命理新論下-p*-*.md', '下冊（研究，第 24–32 章）')]:
    d = BOOKS / '命理新論' / vol
    fs = [p for p in d.glob(pat) if '逐頁' not in p.name]
    if vol == '下冊':
        fs.sort(key=lambda p: int(re.search(r'-p(\d+)-', p.name).group(1)))
    else:
        fs.sort()
    v = Node('命理新論 ' + label, 'group', src=d, short=vol)
    v.readme = d / 'README.md' if (d / 'README.md').exists() else None
    v.tsv = None
    v.children = [chapter(p) for p in fs]
    vols.append(v)
mlxl.children = vols

parts = [
    Node('第一部　上課資料', 'part', children=[course, textbook], short='上課資料'),
    Node('第二部　綜合整理', 'part', short='綜合整理', children=[
        chapter(BOOKS / '八字諸書融會貫通指南.md'),
        chapter(BOOKS / '四本書特色比較.md'),
        chapter(BOOKS / '四本書規則-App實作.md'),
    ]),
    Node('第三部　河洛子', 'part', short='河洛子', children=[
        book('量化五行力', '河洛子《八字算命之量化五行力》上、下', short='量化五行力'),
        book('八字真理', '河洛子《算命方程式@八字真理》上、下', short='八字真理'),
        book('滴天髓命例研究', '河洛子《滴天髓命例研究》', short='滴天髓命例研究'),
    ]),
    Node('第四部　陸致極', 'part', short='陸致極', children=[
        book('基礎教程', '陸致極《八字命理學基礎教程》', short='基礎教程'),
        book('動態分析教程', '陸致極《八字命理學動態分析教程》', short='動態分析教程'),
        book('進階教程', '陸致極《八字命理學進階教程》', short='進階教程'),
        book('陸致極-強弱規則', '陸致極 日主強弱規則彙整', short='強弱規則'),
    ]),
    Node('第五部　吳俊民', 'part', short='吳俊民', children=[mlxl]),
    Node('第六部　潘樂德、蘇民峰', 'part', short='潘樂德、蘇民峰', children=[
        book('十神洩天機', '潘樂德《十神洩天機》', short='十神洩天機'),
        book('蘇民峰-八字論命', '蘇民峰《八字論命》', short='八字論命'),
    ]),
    Node('第七部　入門與專題四書', 'part', short='入門四書', children=[
        book('妙算人生八字', '林信銘《妙算人生八字》', short='妙算人生八字'),
        book('輕鬆學算命', '錢思吾《輕鬆學算命》', short='輕鬆學算命'),
        book('八字心訣', '覺慧居士《八字心訣》', short='八字心訣'),
        book('子平正源-轉盤論法', '鄭偉雄《子平正源——轉盤論法》', short='子平正源'),
    ]),
]

# ---------------------------------------------------------------- assign files
seq = [0]
all_nodes = []
src_map = {}      # resolved md path -> node
tail_map = {}     # "dir/file.md" -> node
name_map = {}


def newfile():
    seq[0] += 1
    return f'p{seq[0]:03d}.xhtml'


def walk(n, parent=None):
    n.parent = parent
    n.file = newfile()
    all_nodes.append(n)
    if n.kind == 'chapter':
        src_map[str(n.src.resolve())] = n
        tail_map[n.src.parent.name + '/' + n.src.name] = n
        name_map.setdefault(n.src.name, []).append(n)
    else:
        if getattr(n, 'readme', None):
            src_map[str(n.readme.resolve())] = n
            src_map[str(n.readme.parent.resolve())] = n
            tail_map[n.readme.parent.name + '/README.md'] = n
        elif n.src:
            src_map[str(Path(n.src).resolve())] = n
    for c in n.children:
        walk(c, n)
    if getattr(n, 'tsv', None):
        n.tsvnode = Node(n.short + ' 命例索引', 'cases', src=n.tsv)
        n.tsvnode.parent = n
        n.tsvnode.file = newfile()
        all_nodes.append(n.tsvnode)
        src_map[str(n.tsv.resolve())] = n.tsvnode


index = Node('總索引', 'index'); index.file = 'index.xhtml'
note = chapter(Path('/dev/null/編輯說明.md'), md=EDITOR_NOTE)
note.file = 'note.xhtml'
for p in parts:
    walk(p)

# ---------------------------------------------------------------- markdown -> html


def pandoc(md):
    r = subprocess.run(['pandoc', '-f', 'gfm', '-t', 'html5', '--wrap=none'],
                       input=md.encode(), capture_output=True, check=True)
    return r.stdout.decode()


def parse(htmltext):
    root = lhtml.fragment_fromstring(htmltext or '<p></p>', create_parent='div')
    return root


def heads_of(root):
    out = []
    used = set(e.get('id') for e in root.iter() if isinstance(e.tag, str) and e.get('id'))
    n = 0
    for e in root.iter('h1', 'h2', 'h3', 'h4', 'h5', 'h6'):
        if not e.get('id'):
            n += 1
            while f's{n}' in used:
                n += 1
            e.set('id', f's{n}'); used.add(f's{n}')
        out.append((int(e.tag[1]), clean(e.text_content()), e.get('id')))
    return out


images = {}
for n in all_nodes:
    if n.kind == 'chapter':
        n.body = parse(pandoc(n.md))
        n.heads = heads_of(n.body)
    elif n.kind in ('book', 'group') and getattr(n, 'readme', None):
        n.body = parse(pandoc(n.readme.read_text(encoding='utf-8')))
        for h in n.body.iter('h1'):
            h.tag = 'h2'  # README's own title becomes a sub-heading under the generated page title
            h.text_content()
            break
        n.heads = heads_of(n.body)
note.body = parse(pandoc(note.md)); note.heads = heads_of(note.body)

lesson_by_date = {n.src.stem: n for n in course.children if re.match(r'2026-\d\d-\d\d$', n.src.stem)}


def resolve(n, href):
    """Return new href (relative to text/), or ('img', path), or None to unwrap."""
    if re.match(r'^[a-z]+:', href):
        return href
    path, _, frag = href.partition('#')
    if not path:
        if n.kind == 'chapter' and frag in lesson_by_date and n.src.name == '課程總覽.md':
            return lesson_by_date[frag].file
        return href
    base = n.src if n.kind != 'chapter' else n.src.parent
    if n.kind in ('book', 'group') and getattr(n, 'readme', None):
        base = n.readme.parent
    from urllib.parse import unquote
    path = unquote(path)
    p = (Path(base) / path).resolve()
    if p.suffix.lower() in ('.png', '.jpg', '.jpeg') and p.exists():
        return ('img', p)
    t = src_map.get(str(p))
    if t is None:
        t = tail_map.get(p.parent.name + '/' + p.name)
    if t is None and p.name in name_map and len(name_map[p.name]) == 1 and p.suffix == '.md':
        t = name_map[p.name][0]
    if t is None:
        m = re.search(r'(2026-\d\d-\d\d)-lesson\.md$', p.name)
        if m and m.group(1) in lesson_by_date:
            t = lesson_by_date[m.group(1)]
    if t is None:
        return None
    if frag and any(h[2] == frag for h in t.heads):
        return t.file + '#' + frag
    return t.file


unresolved = []


def fix_links(n):
    if n.body is None:
        return
    for a in list(n.body.iter('a')):
        href = a.get('href')
        if not href:
            continue
        r = resolve(n, href)
        if isinstance(r, tuple):
            p = r[1]
            name = 'img' + str(len(images) + 1) + p.suffix.lower()
            for k, v in images.items():
                if v == p:
                    name = k
            images[name] = p
            img = etree.Element('img', src='../images/' + name, alt=a.text_content() or name)
            img.set('class', 'photo')
            a.text = None
            for c in list(a):
                a.remove(c)
            a.append(img)
            a.set('href', '../images/' + name)
            a.tag = 'span'
            del a.attrib['href']
        elif r is None:
            unresolved.append((n.title, href))
            a.tag = 'span'
            del a.attrib['href']
        else:
            a.set('href', r)


for n in all_nodes + [note]:
    fix_links(n)

# ---------------------------------------------------------------- page generation
CSS = """
body { font-family: serif; line-height: 1.65; margin: 0 .4em; }
h1 { font-size: 1.45em; margin: .6em 0 .5em; border-bottom: 2px solid #b33; padding-bottom: .2em; }
h2 { font-size: 1.25em; margin: 1.2em 0 .4em; border-left: 4px solid #b33; padding-left: .35em; }
h3 { font-size: 1.1em; margin: 1em 0 .3em; }
h4, h5, h6 { font-size: 1em; margin: .8em 0 .3em; }
p { margin: .4em 0; }
table { border-collapse: collapse; margin: .5em 0; font-size: .92em; }
th, td { border: 1px solid #999; padding: .15em .4em; vertical-align: top; }
th { background: #eee; }
sub { font-size: .62em; vertical-align: sub; line-height: 0; }
sup { font-size: .62em; line-height: 0; }
blockquote { margin: .5em 0 .5em .4em; padding-left: .6em; border-left: 3px solid #bbb; color: #444; }
code { font-family: monospace; font-size: .9em; }
pre { white-space: pre-wrap; font-size: .85em; }
img.photo { max-width: 100%; display: block; margin: .4em 0; }
.mtoc { font-size: .9em; border: 1px solid #ccc; padding: .3em .8em; margin: .6em 0 1em; }
.mtoc p.t { font-weight: bold; margin: .2em 0; }
.mtoc ul { margin: .1em 0; padding-left: 1.2em; }
.crumb { font-size: .8em; color: #777; margin-top: .3em; }
.crumb a { color: #777; }
ul.idx { padding-left: 1.1em; }
ul.idx > li { margin: .15em 0; }
.small { font-size: .85em; color: #555; }
"""


def esc(s):
    return html.escape(s, quote=True)


def ser(el):
    return etree.tostring(el, method='xml', encoding='unicode')


def body_inner(root):
    s = esc(root.text) if root.text else ''
    return s + ''.join(ser(c) for c in root)


def page(title, inner):
    return f"""<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops" lang="zh-Hant" xml:lang="zh-Hant">
<head><meta charset="utf-8"/><title>{esc(title)}</title><link rel="stylesheet" type="text/css" href="../style.css"/></head>
<body>
{inner}
</body>
</html>
"""


def crumb(n):
    chain = []
    p = n.parent
    while p is not None:
        chain.append(p)
        p = p.parent
    items = ['<a href="index.xhtml">總索引</a>'] + [f'<a href="{x.file}">{esc(x.short)}</a>' for x in reversed(chain)]
    return '<p class="crumb">' + ' › '.join(items) + '</p>'


def mini_toc(n, maxlevel=3):
    hs = [h for h in n.heads if 2 <= h[0] <= maxlevel]
    if len([h for h in hs if h[0] == 2]) < 2 and len(hs) < 4:
        return ''
    out, cur = ['<div class="mtoc"><p class="t">本章目錄</p><ul>'], 2
    open_sub = False
    for lv, t, i in hs:
        if lv == 2:
            if open_sub:
                out.append('</ul></li>'); open_sub = False
            elif len(out) > 1:
                out.append('</li>')
            out.append(f'<li><a href="#{esc(i)}">{esc(t)}</a>')
        else:
            if not open_sub:
                out.append('<ul>'); open_sub = True
            out.append(f'<li><a href="#{esc(i)}">{esc(t)}</a></li>')
    if open_sub:
        out.append('</ul></li>')
    elif len(out) > 1:
        out.append('</li>')
    out.append('</ul></div>')
    s = ''.join(out)
    return s.replace('<ul></li>', '</li>') if False else s


def chapter_page(n):
    root = n.body
    h1 = root.find('h1')
    toc = mini_toc(n)
    inner = crumb(n)
    if h1 is not None and toc:
        idx = list(root).index(h1)
        before = ''.join(ser(c) for c in list(root)[:idx + 1])
        after = ''.join(ser(c) for c in list(root)[idx + 1:])
        inner += (esc(root.text) if root.text else '') + before + toc + after
    else:
        inner += body_inner(root)
    return page(n.title, inner)


def chapter_list(n, with_h2=True, depth=0):
    out = ['<ul class="idx">']
    for c in n.children:
        if c.kind == 'chapter':
            out.append(f'<li><a href="{c.file}">{esc(c.title)}</a>')
            h2 = [h for h in c.heads if h[0] == 2]
            if with_h2 and h2:
                out.append('<ul class="small">' + ''.join(
                    f'<li><a href="{c.file}#{esc(i)}">{esc(t)}</a></li>' for _, t, i in h2) + '</ul>')
            out.append('</li>')
        else:
            out.append(f'<li><a href="{c.file}"><b>{esc(c.title)}</b></a>')
            out.append(chapter_list(c, with_h2=False))
            out.append('</li>')
    if getattr(n, 'tsvnode', None):
        out.append(f'<li><a href="{n.tsvnode.file}">{esc(n.tsvnode.title)}</a></li>')
    out.append('</ul>')
    return ''.join(out)


def container_page(n):
    inner = crumb(n) + f'<h1 id="top">{esc(n.title)}</h1>'
    inner += '<h2 id="toc-list">章節索引</h2>' + chapter_list(n, with_h2=(n.kind != 'part'))
    if n.body is not None:
        inner += '<h2 id="about">本書說明</h2><div class="readme">' + body_inner(n.body) + '</div>'
    return page(n.title, inner)


def cases_page(n):
    b = n.parent
    files = {c.src.name: c for c in all_nodes if c.kind == 'chapter' and c.src.parent == b.src}
    rows = list(csv.DictReader(open(n.src, encoding='utf-8'), delimiter='\t'))
    out = [crumb(n), f'<h1 id="top">{esc(n.title)}</h1>',
           f'<p class="small">共 {len(rows)} 例。四柱依「年 月 日 時」排列（與 App 排盤表的「時 日 月 年」方向相反），點命例標題跳到筆記。</p>',
           '<table><thead><tr><th>編號</th><th>頁</th><th>命例</th><th>年 月 日 時</th></tr></thead><tbody>']
    miss = 0
    for r in rows:
        c = files.get(r.get('file', ''))
        title = r.get('title', '')
        href = None
        m = re.match(r'(\d+)-(\d+)-(\d+)$', r.get('id', ''))
        if not r.get('file') and m:
            ch, sec, k = map(int, m.groups())
            c = next((x for x in files.values() if x.src.name.startswith(f'第{ch}章')), None)
            title = '；'.join(x for x in (r.get('book_pattern', ''), r.get('note', '')) if x)
            if c:
                href = c.file
                si = ki = 0
                for lv, t, i in c.heads:
                    if lv == 2 and re.match(r'第.節', t):
                        si += 1; ki = 0
                    elif si == sec and re.match(r'例', t):
                        ki += 1
                        if ki == k:
                            href += '#' + i; title = t; break
                else:
                    miss += 1
            c = None
        if c:
            href = c.file
            key = clean(title)
            cand = [h for h in c.heads if h[1] == key] or \
                   [h for h in c.heads if key and (key in h[1] or h[1] in key) and len(h[1]) > 3]
            if not cand:
                k2 = re.split(r'[：:（(]', key)[0].strip()
                cand = [h for h in c.heads if k2 and k2 in h[1]]
            if cand:
                href += '#' + cand[0][2]
            else:
                miss += 1
        pillars = ' '.join(r.get(k, '') for k in ('year', 'month', 'day', 'hour'))
        t = f'<a href="{href}">{esc(title)}</a>' if href else esc(title)
        out.append(f'<tr><td style="white-space:nowrap">{esc(r.get("id",""))}</td><td>{esc(r.get("page",""))}</td><td>{t}</td><td style="white-space:nowrap">{esc(pillars)}</td></tr>')
    out.append('</tbody></table>')
    n.miss = miss
    return page(n.title, ''.join(out))


def index_page():
    out = ['<h1 id="top">八字學習筆記合集・總索引</h1>',
           f'<p class="small">許羽賢老師課程與各本八字書的整理筆記，{TODAY} 版。閱讀器目錄可展開到「節」；<a href="note.xhtml">編輯說明</a>寫了收錄範圍與取捨。</p>']
    for p in parts:
        out.append(f'<h2 id="{p.file[:-6]}"><a href="{p.file}">{esc(p.title)}</a></h2><ul class="idx">')
        for b in p.children:
            if b.kind == 'chapter':
                out.append(f'<li><a href="{b.file}">{esc(b.title)}</a></li>')
                continue
            out.append(f'<li><a href="{b.file}"><b>{esc(b.title)}</b></a>')
            out.append(chapter_list(b, with_h2=False))
            out.append('</li>')
        out.append('</ul>')
    return page('總索引', ''.join(out))


# ---------------------------------------------------------------- nav + ncx

def nav_items(n, depth):
    t = esc(n.title)
    kids = []
    if n.kind == 'chapter':
        kids = [f'<li><a href="text/{n.file}#{esc(i)}">{esc(tt)}</a></li>' for lv, tt, i in n.heads if lv == 2]
    else:
        kids = [nav_items(c, depth + 1) for c in n.children]
        if getattr(n, 'tsvnode', None):
            kids.append(f'<li><a href="text/{n.tsvnode.file}">{esc(n.tsvnode.title)}</a></li>')
    s = f'<li><a href="text/{n.file}">{t}</a>'
    if kids:
        s += '<ol>' + ''.join(kids) + '</ol>'
    return s + '</li>'


def ncx_points(n, counter):
    counter[0] += 1
    s = f'<navPoint id="n{counter[0]}"><navLabel><text>{esc(n.title)}</text></navLabel><content src="text/{n.file}"/>'
    if n.kind == 'chapter':
        for lv, tt, i in n.heads:
            if lv == 2:
                counter[0] += 1
                s += f'<navPoint id="n{counter[0]}"><navLabel><text>{esc(tt)}</text></navLabel><content src="text/{n.file}#{esc(i)}"/></navPoint>'
    else:
        for c in n.children:
            s += ncx_points(c, counter)
        if getattr(n, 'tsvnode', None):
            counter[0] += 1
            s += f'<navPoint id="n{counter[0]}"><navLabel><text>{esc(n.tsvnode.title)}</text></navLabel><content src="text/{n.tsvnode.file}"/></navPoint>'
    return s + '</navPoint>'


BOOK_TITLE = '八字學習筆記合集'
uid = 'urn:uuid:' + str(uuid.uuid5(uuid.NAMESPACE_URL, 'bazi-notes-collection'))

pages = [('index.xhtml', index_page()), ('note.xhtml', page('編輯說明', crumb(note) + body_inner(note.body)))]
for n in all_nodes:
    if n.kind == 'chapter':
        pages.append((n.file, chapter_page(n)))
    elif n.kind == 'cases':
        pages.append((n.file, cases_page(n)))
    else:
        pages.append((n.file, container_page(n)))

nav = f"""<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops" lang="zh-Hant" xml:lang="zh-Hant">
<head><meta charset="utf-8"/><title>目錄</title><link rel="stylesheet" type="text/css" href="style.css"/></head>
<body>
<nav epub:type="toc" id="toc"><h1>目錄</h1><ol>
<li><a href="text/index.xhtml">總索引</a></li>
<li><a href="text/note.xhtml">編輯說明</a></li>
{''.join(nav_items(p, 0) for p in parts)}
</ol></nav>
<nav epub:type="landmarks" hidden=""><ol>
<li><a epub:type="toc" href="text/index.xhtml">總索引</a></li>
<li><a epub:type="bodymatter" href="text/{parts[0].file}">本文</a></li>
</ol></nav>
</body></html>
"""
cnt = [0]
pts = ''
for extra, f in (('總索引', 'index.xhtml'), ('編輯說明', 'note.xhtml')):
    cnt[0] += 1
    pts += f'<navPoint id="n{cnt[0]}"><navLabel><text>{extra}</text></navLabel><content src="text/{f}"/></navPoint>'
pts += ''.join(ncx_points(p, cnt) for p in parts)
ncx = f"""<?xml version="1.0" encoding="utf-8"?>
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" version="2005-1" xml:lang="zh-Hant">
<head><meta name="dtb:uid" content="{uid}"/><meta name="dtb:depth" content="6"/></head>
<docTitle><text>{BOOK_TITLE}</text></docTitle>
<navMap>{pts}</navMap></ncx>
"""

manifest = ['<item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>',
            '<item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>',
            '<item id="css" href="style.css" media-type="text/css"/>']
spine = []
for f, _ in pages:
    iid = 'x_' + f[:-6]
    manifest.append(f'<item id="{iid}" href="text/{f}" media-type="application/xhtml+xml"/>')
    spine.append(f'<itemref idref="{iid}"/>')
for name in images:
    mt = 'image/png' if name.endswith('.png') else 'image/jpeg'
    manifest.append(f'<item id="i_{name.replace(".", "_")}" href="images/{name}" media-type="{mt}"/>')

modified = datetime.datetime.now(datetime.timezone.utc).strftime('%Y-%m-%dT%H:%M:%SZ')
opf = f"""<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="uid" xml:lang="zh-Hant">
<metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
<dc:identifier id="uid">{uid}</dc:identifier>
<dc:title>{BOOK_TITLE}</dc:title>
<dc:language>zh-Hant</dc:language>
<dc:creator>PaiYi 的學習筆記</dc:creator>
<dc:description>許羽賢老師子平八字課程筆記與各本八字書整理筆記合集（自行整理，非原書內容）。</dc:description>
<dc:date>{TODAY}</dc:date>
<meta property="dcterms:modified">{modified}</meta>
</metadata>
<manifest>
{chr(10).join(manifest)}
</manifest>
<spine toc="ncx">
{chr(10).join(spine)}
</spine>
</package>
"""

OUT.parent.mkdir(parents=True, exist_ok=True)
with zipfile.ZipFile(OUT, 'w') as z:
    z.writestr(zipfile.ZipInfo('mimetype'), 'application/epub+zip', compress_type=zipfile.ZIP_STORED)
    z.writestr('META-INF/container.xml', '<?xml version="1.0"?><container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container"><rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/></rootfiles></container>', compress_type=zipfile.ZIP_DEFLATED)
    z.writestr('OEBPS/content.opf', opf, compress_type=zipfile.ZIP_DEFLATED)
    z.writestr('OEBPS/nav.xhtml', nav, compress_type=zipfile.ZIP_DEFLATED)
    z.writestr('OEBPS/toc.ncx', ncx, compress_type=zipfile.ZIP_DEFLATED)
    z.writestr('OEBPS/style.css', CSS, compress_type=zipfile.ZIP_DEFLATED)
    for f, content in pages:
        z.writestr('OEBPS/text/' + f, content, compress_type=zipfile.ZIP_DEFLATED)
    for name, p in images.items():
        z.write(p, 'OEBPS/images/' + name, compress_type=zipfile.ZIP_DEFLATED)

chap = [n for n in all_nodes if n.kind == 'chapter']
print('pages', len(pages), 'chapters', len(chap), 'books', len([n for n in all_nodes if n.kind == 'book']),
      'images', len(images), 'nav-sections', sum(len([h for h in n.heads if h[0] == 2]) for n in chap))
for n in all_nodes:
    if n.kind == 'cases':
        print('cases', n.title, 'unmatched anchors', n.miss)
print('unresolved links', len(unresolved))
for t, h in unresolved[:40]:
    print('  ', t[:30], '->', h)
print('size', OUT.stat().st_size)
