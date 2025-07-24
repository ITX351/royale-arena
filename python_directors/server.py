from flask import Flask
from flask import request, redirect, url_for, render_template
from user import users, encrypt, decrypt, is_director, has_user, get_id
from data import roles, places, alive_roles, enabled_places, items, all_items, globals
from action import act, act_admin
from constants import *
import message

app = Flask(__name__)

@app.route('/hello')
def hello_world():
    return 'Hello, World!'

@app.route('/')
def index():
    return render_template('login.html')

@app.route('/api/login', methods=['POST'])
def login():
    id = request.form.get('id', '')
    return redirect(url_for('view_messages', key=encrypt(id)))

@app.route('/api/view/<key>')
def view_messages(key):
    try:
        user = decrypt(key)
        if not has_user(user):
            raise
    except:
        return redirect(url_for('index'))

    if is_director(user):
        messages = message.all()
        return render_template('view_all.html', messages=messages, send_url='/api/send/'+key, admin_url='/api/admin/'+key,
                               roles=alive_roles(), places=enabled_places(), items=items, globals=globals)

    else:
        messages = message.get(user)
        role = roles[users[user]]
        return render_template('view.html', messages=messages, send_url='/api/send/'+key, role=role, all_roles=roles.keys(),
                               roles=alive_roles(), places=enabled_places(), items=items, all_items=list(set(all_items())), item_levels = ITEM_TARGET_LEVELS)

@app.route('/api/send/<key>', methods=['POST'])
def send_message(key):
    try:
        user = decrypt(key)
        if not has_user(user):
            raise
    except:
        return redirect(url_for('index'))

    action = request.form.get('action', '')
    if action != 'send':
        role = roles[users[user]]
        content = act(role, action, request.form)
        message.add(user, 2, content)
    else:
        content = request.form.get('content', '')
        if len(content) > 0:
            if is_director(user):
                dst = request.form.get('to', '')
                message.add(get_id(dst), 1, content)
            else:
                message.add(user, 0, content)
    return redirect(url_for('view_messages', key=key))

@app.route('/api/admin/<key>', methods=['POST'])
def admin_action(key):
    try:
        user = decrypt(key)
        if not has_user(user) or not is_director(user):
            raise
    except:
        return redirect(url_for('index'))
    
    action = request.form.get('action', '')
    if action == 'jump':
        return redirect(url_for('view_messages', key=encrypt(get_id(request.form.get('jump_target', '')))))
    act_admin(action, request.form)
    return redirect(url_for('view_messages', key=key))
