import threading
import random
import traceback
import time
from data import roles, places, items, globals, item_generated
from user import get_id
from constants import *
import message

feedbacks = {}
mutex = threading.Lock()
living = False

def move(role, to):
    if to not in places:
        return '移动失败'
    dst = places[to]
    if not dst['able']:
        return '移动失败'
    if to == role['location']:
        return '无需移动'
    src = places[role['location']]
    role['location'] = to
    src['exists'].remove(role['name'])
    dst['exists'].append(role['name'])
    role['strength'] -= COSTS[MOVE]
    return '移动成功'

def search(role):
    feedbacks[role['name']] = ''
    place = places[role['location']]
    if len(place['exists']) == 1:
        role['strength'] -= COSTS[SEARCH]
        return '空'
    res = random.choice(list(filter(lambda x: x != role['name'], place['exists'])))
    feedbacks[role['name']] = res
    role['strength'] -= COSTS[SEARCH]
    if res in roles and random.random() > globals['weather']:
        res = '未知角色'
    return res

def can_pick(role, fb):
    sort14 = len(list(filter(lambda x: x in items and items[x][0] <= 4, role['things'])))
    sort59 = len(list(filter(lambda x: x not in items or items[x][0] >= 5, role['things'])))
    if sort14 + sort59 >= ITEM_CAPACITY:
        return 3 # 道具过多
    if sort14 >= ITEM_WEAPON_CAPACITY and (fb in items and items[fb][0] <= 4):
        return 1 # 武器过多
    if sort59 >= ITEM_OTHERS_CAPACITY and (fb not in items or items[fb][0] >= 5):
        return 2 # 非武器类道具过多
    return 0 # OK

def pick(role):
    fb = feedbacks.get(role['name'], '')
    feedbacks[role['name']] = ''
    place = places[role['location']]
    if fb == '' or fb in roles or fb not in place['exists']:
        return '捡拾失败'
    validation = can_pick(role, fb)
    if validation == 3:
        feedbacks[role['name']] = fb
        return '最多拥有' + str(ITEM_CAPACITY) + '件道具，请丢弃一件后重新捡拾'
    if validation == 1:
        feedbacks[role['name']] = fb
        return '最多拥有' + str(ITEM_WEAPON_CAPACITY) + '件武器，请丢弃一件现有武器后重新捡拾'
    if validation == 2:
        feedbacks[role['name']] = fb
        return '最多拥有' + str(ITEM_OTHERS_CAPACITY) + '件非武器类道具，请丢弃一件后重新捡拾'
    place['exists'].remove(fb)
    role['things'].append(fb)
    role['strength'] -= COSTS[PICK]
    return '捡拾成功'

def be_attacked(source, target, value, ignore_defend=False, continual=False, method='不明原因'):
    if not ignore_defend:
        if set(ITEM_PROTECT) & set(target['hands']):
            value -= ITEM_PROTECT_VALUE
        if set(ITEM_ENSURE) & set(target['hands']):
            value //= 2
    if continual:
        target['injured'] = ITEM_HOT_AOE_DAMAGE_LASTED
    target['life'] -= value
    message.add(get_id(target['name']), 1, '你被攻击，受到' + str(value) + '伤害')
    if target['life'] <= 0:
        target['life'] = 0
        target['able'] = False
        places[target['location']]['exists'].remove(target['name'])
        random.shuffle(target['things'])
        gain = []
        drop = []
        for fb in target['things']:
            if can_pick(source, fb) == 0:
                gain.append(fb)
                source['things'].append(fb)
            else:
                drop.append(fb)
                places[target['location']]['exists'].append(fb)
        ret = '杀死了' + target['name'] + '，死因为' + method + '，'
        if len(gain) > 0 and len(drop) == 0:
            ret += '获得全部道具：' + ' '.join(gain)
        elif len(gain) > 0 and len(drop) > 0:
            ret += '获得道具：' + ' '.join(gain) + '，原地掉落道具：' + ' '.join(drop)
        elif len(gain) == 0 and len(drop) > 0:
            ret += '背包已满，原地掉落道具：' + ' '.join(drop)
        else:
            ret += '对方无道具'
        target['things'] = []
        target['hands'] = []
        return ret
    return ''

def do_attack(role, target):
    res = []
    if set(ITEM_BOMB) & set(role['hands']):
        for bomb in ITEM_BOMB:
            if bomb in role['hands']:
                for t in places[role['location']]['exists'][:]:
                    if t in roles and t != role['name']:
                        res.append(be_attacked(role, roles[t], ITEM_BOMB_DAMAGE + ORDINARY_DAMAGE, ignore_defend=True))
                role['hands'].remove(bomb)
                role['things'].remove(bomb)
                break
    elif set(ITEM_KILL) & set(role['hands']):
        for kill in ITEM_KILL:
            if kill in role['hands']:
                res.append(be_attacked(role, target, 100 + ORDINARY_DAMAGE, ignore_defend=True))
                role['hands'].remove(kill)
                role['things'].remove(kill)
                break
    else:
        dec = ORDINARY_DAMAGE
        district = False
        method = '普通攻击'
        weapons = list(filter(lambda x: items.get(x, [9, 1])[0] <= 4 and items.get(x, [9, 1])[1] > 0, role['hands']))
        if len(weapons) > 0:
            weapon = weapons[0]
            items[weapon][1] -= 1
            if items[weapon][1] == 0:
                role['hands'].remove(weapon)
                role['things'].remove(weapon)
            if items[weapon][0] == 1:
                dec += ITEM_HOT_AOE_DAMAGE
                method = '致命伤害'
                district = True
            elif items[weapon][0] == 2:
                dec += ITEM_HOT_VITAL_DAMAGE
                method = '致命伤害'
            elif items[weapon][0] == 3:
                dec += ITEM_HOT_DAMAGE
                method = '致命伤害'
            else:
                dec += ITEM_COLD_DAMAGE
                method = '失血过多'
        if district:
            for t in places[role['location']]['exists'][:]:
                if t in roles and t != role['name']:
                    res.append(be_attacked(role, roles[t], dec, continual=True, method=method))
        else:
            res.append(be_attacked(role, target, dec, method=method))
    res = list(filter(lambda x: len(x) > 0, res))
    return '攻击成功。' + '。'.join(res)

def attack(role):
    fb = feedbacks.get(role['name'], '')
    feedbacks[role['name']] = ''
    place = places[role['location']]
    if fb == '' or fb not in roles or fb not in place['exists'] or place['safe']:
        return '攻击失败'
    target = roles[fb]
    if target['life'] <= 0 or target['life'] > 100:
        return '请勿鞭尸'
    res = do_attack(role, target)
    role['strength'] -= COSTS[ATTACK]
    return res

def is_same_type(item1, item2):
    t1 = 1 if items.get(item1, [9, 1])[0] <= 4 else 0
    t2 = 1 if items.get(item2, [9, 1])[0] <= 4 else 0
    return t1 == t2

def equip(role, item):
    if item not in role['things']:
        return '无此道具'
    role['hands'] = list(filter(lambda x: not is_same_type(item, x), role['hands']))
    role['hands'].append(item)
    return '装备成功'

def change_life(role, value):
    death = []
    role['life'] += value
    if role['life'] > 100:
        role['life'] = 100
    if role['life'] <= 0:
        role['life'] = 0
        role['able'] = False
        places[role['location']]['exists'].remove(role['name'])
        places[role['location']]['exists'] += role['things']
        role['hands'] = []
        role['things'] = []
        death.append(role['name'])
    message.add(get_id(role['name']), 1, '你的生命值变化了' + str(value))
    return death

def get_occupation(name):
    if '<' not in name or '>' not in name:
        return ''
    l = name.find('<')
    r = name.find('>')
    return name[l + 1: r]

def can_generate(item):
    # 无次数限制，或者尚未产生过
    return items.get(item, [9, 1])[1] > 10 or item not in item_generated

def upgrade(role, item, target):
    formulas = ITEM_UPGRADE_MAP.get(item, [])
    random.shuffle(formulas)
    target_formula = []
    # 先寻找符合目标等级的公式合成
    for formula in formulas:
        if target in formula[0]:
            if set(formula[1:]) <= set(role['things']) and can_generate(formula[0]):
                target_formula = formula
                break
    # 寻找其它公式合成
    if len(target_formula) == 0:
        for formula in formulas:
            if set(formula[1:]) <= set(role['things']) and can_generate(formula[0]):
                target_formula = formula
                break
    if len(target_formula) == 0:
        return '合成失败，所需材料不足'
    for x in target_formula[1:]:
        role['things'].remove(x)
        if x in role['hands']:
            role['hands'].remove(x)
    role['hands'].remove(item)
    role['things'].remove(item)
    role['things'].append(target_formula[0])
    equip(role, target_formula[0])
    item_generated.add(target_formula[0])
    return '合成并装备' + target_formula[0] + '成功，消耗了' + '、'.join([item] + target_formula[1:])

def use(role, item, target):
    if item not in role['hands']:
        return '未装备此道具'
    item_occupation = get_occupation(item)
    role_occupation = get_occupation(role['name'])
    if item_occupation not in role_occupation:
        return '使用失败，道具职业与角色职业不匹配'
    tp = items.get(item, [9, 1])[0]
    times = items.get(item, [9, 1])[1]
    if times <= 0:
        return '当夜次数已用尽'
    if tp <= 5:
        return '1~5类道具无需使用，拿在手里以后，直接搜索、攻击即可'
    elif tp <= 7:
        items[item][1] -= 1
        if item in ITEM_LOCATOR:
            if target == '':
                items[item][1] += 1
                return '使用失败，未选择使用目标'
            for k, v in places.items():
                if v['able'] and target in v['exists']:
                    return k
            for k, v in roles.items():
                if target in v['things']:
                    return k + ' ' + v['location']
            return '场上无此道具'
        elif item in ITEM_TELESCOPE:
            if target == '':
                items[item][1] += 1
                return '使用失败，未选择使用目标'
            return ' '.join(roles[target]['things']) if len(roles[target]['things']) > 0 else '对方无道具'
        elif item in ITEM_LOCK:
            for t in places[role['location']]['exists'][:]:
                if t in roles and t != role['name']:
                    roles[t]['able'] = False
            return '使用成功'
        else:
            return item + '请QQ联系导演，下一日公示'
    elif tp == 8:
        death = []
        if item in ITEM_KILL:
            death += change_life(roles[target], -100)
        elif item in ITEM_BOMB:
            for t in places[role['location']]['exists'][:]:
                if t in roles and t != role['name']:
                    death += change_life(roles[t], -ITEM_BOMB_DAMAGE)
        role['hands'].remove(item)
        role['things'].remove(item)
        return '使用成功。' + ('杀死了' + ' '.join(death) + '，道具散落' if len(death) > 0 else '无人死亡')
    elif tp == 9:
        if item in ITEM_TEA:
            role['strength'] += 20
            if role['strength'] > 100:
                role['strength'] = 100
        elif item in ITEM_WATER:
            role['strength'] += 50
            if role['strength'] > 100:
                role['strength'] = 100
        elif item in ITEM_ACTIVE:
            role['strength'] = 100
        elif item in ITEM_BANDAGE:
            if role['injured'] > 0:
                role['injured'] = 0
            else:
                role['life'] += 10
                if role['life'] > 100:
                    role['life'] = 100
        elif item in ITEM_CURE:
            if role['injured'] > 0:
                role['injured'] = 0
            else:
                role['life'] += 30
                if role['life'] > 100:
                    role['life'] = 100
        elif item in ITEM_PREMIUM_CURE:
            role['injured'] = 0
            role['life'] += 50
            if role['life'] > 100:
                role['life'] = 100
        elif item in ITEM_PERFECT_CURE:
            role['injured'] = 0
            role['life'] = 100
        elif item in ITEM_UPGRADE_MAP:
            return upgrade(role, item, target)
        role['hands'].remove(item)
        role['things'].remove(item)
        return '使用成功'
    return '未定义的道具'

def throw(role, item):
    if item not in role['things']:
        return '无此道具'
    role['things'].remove(item)
    if item in role['hands']:
        role['hands'].remove(item)
    places[role['location']]['exists'].append(item)
    return '已丢弃在原地'

def deliver(role, target, content):
    res = ''
    if len(content) > 0 and len(target) > 0 or target not in roles:
        if role['deliver'] == 0:
            role['deliver'] = 1
            role['strength'] -= 5
            message.add(get_id(target), 1, role['name'] + '传音给你：' + content)
            res = '传音成功'
        else:
            res = '传音失败，当夜次数已用尽'
    else:
        res = '传音失败，无效的对象或消息'
    return res

def born(role, place):
    if role['location'] == '' and place in places:
        role['location'] = place
        try:
            places[place]['exists'].remove(role['name'])
        except:
            pass
        places[place]['exists'].append(role['name'])
        return '成功出生在' + place
    else:
        return '你已经选择过出生地，或所选地点不存在'

def act(role, action, params):
    if action == 'born':
        return born(role, params.get('born_in', ''))
    global living
    if not living:
        return '行动未开始'
    cost = COSTS.get(action, -1)
    if cost < 0:
        return '未定义的行动'
    if not role['able']:
        return '你被禁止行动'
    if cost > role['strength']:
        return '体力不足'
    if action in [SEARCH]:
        if time.time() - role['ts'] < 30:
            return '两次搜索之间需间隔30秒'
        role['ts'] = time.time()
    mutex.acquire()
    msg = ''
    res = ''
    try:
        if action == MOVE:
            msg = '移动' + params.get('move_to', '')
            res = move(role, params.get('move_to', ''))
        elif action == SEARCH:
            msg = '搜索'
            res = search(role)
        elif action == PICK:
            msg = '捡拾'
            res = pick(role)
        elif action == ATTACK:
            msg = '攻击'
            res = attack(role)
        elif action == EQUIP:
            msg = '装备' + params.get('equip_item', '')
            res = equip(role, params.get('equip_item', ''))
        elif action == USE:
            msg = '使用' + params.get('use_item', '')
            target = ''
            target_role = params.get('use_item_target_role', '')
            if target_role != '':
                msg += '，目标' + target_role
                target = target_role
            target_item = params.get('use_item_target_item', '')
            if target_item != '':
                msg += '，查看' + target_item
                target = target_item
            target_upgrade = params.get('use_item_target_upgrade', '')
            if target_upgrade != '':
                target = target_upgrade
            res = use(role, params.get('use_item', ''), target)
        elif action == THROW:
            msg = '丢弃' + params.get('throw_item', '')
            res = throw(role, params.get('throw_item', ''))
        elif action == DELIVER:
            target = params.get('deliver_target', '')
            content = params.get('deliver_content', '')
            msg = '传音' + target + '：' + content
            res = deliver(role, target, content)
    except Exception as e:
        res = str(e)
        traceback.print_exc()
    if action == MOVE:
        role['rest'] -= 1
    else:
        role['rest'] = 0
    mutex.release()
    return msg + '。反馈：' + res

def act_admin(action, params):
    global living
    res = ''
    mutex.acquire()
    try:
        if action == 'life':
            target = params.get('life_target')
            role = roles[target]
            value = int(params.get('life_value'))
            change_life(role, value)
        elif action == 'strength':
            target = params.get('strength_target')
            value = int(params.get('strength_value'))
            roles[target]['strength'] += value
        elif action == 'rope':
            target = params.get('rope_target')
            roles[target]['able'] = False
        elif action == 'unrope':
            target = params.get('unrope_target')
            roles[target]['able'] = True
        elif action == 'start':
            living = True
        elif action == 'end':
            living = False
            for k, v in roles.items():
                if v['life'] > 0 and v['life'] <= 100:
                    v['able'] = True
                    v['deliver'] = 0
                    v['strength'] += RECOVER_STRENGTH_DAILY
                    if v['strength'] > 100:
                        v['strength'] = 100
                    if v['injured']: # 持续伤害
                        change_life(v, -ITEM_HOT_AOE_DAMAGE_LASTED)
                    if v['rest'] > 0: # 满足静养条件
                        change_life(v, RECOVER_LIFE_RESTED)
                        v['strength'] = 100
                    v['rest'] = 2
            for i in ITEM_LOCATOR + ITEM_LOCK + ITEM_SHOW:
                items[i][1] = 1
            for i in ITEM_TELESCOPE:
                items[i][1] = 2
        elif action == 'save':
            print(places)
            print(roles)
            print(items)
            print(message.messages)
        elif action == 'drop':
            what = params.get('drop_item')
            where = params.get('drop_place')
            places[where]['exists'].append(what)
        elif action == 'destroy':
            target = params.get('destroy_place')
            for suf in [''] + DIRECTIONS:
                if target + suf in places:
                    places[target + suf]['able'] = False
                    places[target + suf]['exists'] = []
        elif action == MOVE:
            who = params.get('move_target')
            where = params.get('move_place')
            try:
                dst = places[where]
                src = places[roles[who]['location']]
                roles[who]['location'] = where
                src['exists'].remove(who)
                dst['exists'].append(who)
            except:
                pass
        elif action == 'give':
            who = params.get('give_target')
            what = params.get('give_item')
            try:
                if what in roles[who]['things']:
                    roles[who]['things'].remove(what)
                    roles[who]['hands'].remove(what)
                else:
                    roles[who]['things'].append(what)
            except:
                pass
        elif action == 'born':
            for role in roles.values():
                if role['location'] == '':
                    born(role, random.choice(list(places.keys())))
        elif action == 'vote':
            vote = params.get('vote_target')
            places[roles[vote]['location']]['exists'] += roles[vote]['things']
            roles[vote]['things'] = []
            roles[vote]['hands'] = []
        elif action == 'weather':
            globals['weather'] = float(params.get('weather_new', globals['weather']))
    except Exception as e:
        res = str(e)
        traceback.print_exc()
    mutex.release()
