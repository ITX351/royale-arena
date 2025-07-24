from constants import *
import base64

users = {
    DIRECTOR_PASSWORD: '导演',
    AUDIENCE_PASSWORD: '观众'
}

for i in range(0, len(ROLES)):
    users[PASSWORDS[i]] = ROLES[i]

user_ids = {
    '导演': DIRECTOR_PASSWORD,
    '观众': AUDIENCE_PASSWORD
}

for i in range(0, len(ROLES)):
    user_ids[ROLES[i]] = PASSWORDS[i]

def encrypt(key):
    return base64.urlsafe_b64encode(key.encode('ascii')).decode('ascii')

def decrypt(key):
    return base64.urlsafe_b64decode(key.encode('ascii')).decode('ascii')

def has_user(user):
    return user in users

def is_director(user):
    return user == '导演' or users.get(user, '') == '导演'

def get_id(name):
    name = name.strip()
    return user_ids.get(name, '')
