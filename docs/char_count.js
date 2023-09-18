const CHAR_COUNT = {
    "我": 516,
    "彼": 442,
    "之": 366,
    "此": 348,
    "於": 336,
    "在": 318,
    "行": 304,
    "言": 274,
    "汝": 265,
    "心": 253,
    "処": 213,
    "無": 212,
    "何": 210,
    "善": 178,
    "極": 177,
    "終": 176,
    "学": 174,
    "等": 149,
    "銭": 143,
    "人": 139,
    "識": 119,
    "乎": 115,
    "時": 103,
    "来": 98,
    "与": 97,
    "家": 94,
    "一": 92,
    "書": 90,
    "日": 88,
    "国": 84,
    "下": 81,
    "手": 78,
    "労": 74,
    "口": 72,
    "力": 72,
    "目": 71,
    "水": 67,
    "多": 67,
    "而": 64,
    "生": 59,
    "上": 56,
    "噫": 52,
    "使": 51,
    "須": 49,
    "友": 49,
    "子": 48,
    "為": 47,
    "星": 46,
    "筆": 45,
    "風": 44,
    "如": 44,
    "哩": 44,
    "牌": 43,
    "天": 42,
    "機": 41,
    "道": 38,
    "木": 38,
    "大": 38,
    "加": 37,
    "火": 35,
    "男": 34,
    "周": 34,
    "冠": 34,
    "再": 34,
    "全": 34,
    "茶": 33,
    "別": 32,
    "其": 32,
    "絵": 31,
    "二": 31,
    "門": 30,
    "車": 29,
    "小": 29,
    "故": 28,
    "入": 28,
    "集": 27,
    "果": 27,
    "女": 27,
    "甘": 26,
    "互": 26,
    "足": 25,
    "勿": 25,
    "件": 25,
    "術": 24,
    "真": 24,
    "清": 24,
    "闇": 23,
    "輩": 23,
    "衣": 23,
    "硬": 23,
    "耳": 22,
    "寝": 22,
    "字": 22,
    "同": 22,
    "受": 22,
    "神": 21,
    "父": 21,
    "地": 21,
    "即": 21,
    "種": 20,
    "犬": 20,
    "律": 20,
    "淮": 19,
    "川": 19,
    "声": 19,
    "刀": 19,
    "長": 18,
    "遊": 18,
    "物": 18,
    "激": 18,
    "戦": 18,
    "夏": 18,
    "助": 18,
    "倉": 18,
    "杯": 17,
    "少": 17,
    "始": 17,
    "雪": 16,
    "訴": 16,
    "片": 16,
    "母": 16,
    "十": 16,
    "魚": 15,
    "酒": 15,
    "机": 15,
    "急": 15,
    "奮": 15,
    "壊": 15,
    "享": 15,
    "鳥": 14,
    "草": 14,
    "花": 14,
    "悪": 14,
    "付": 14,
    "石": 13,
    "値": 13,
    "軟": 12,
    "蜜": 12,
    "色": 12,
    "美": 12,
    "箱": 12,
    "怖": 12,
    "囲": 12,
    "春": 11,
    "新": 11,
    "席": 11,
    "失": 11,
    "三": 11,
    "貧": 10,
    "豊": 10,
    "琴": 10,
    "将": 10,
    "定": 10,
    "体": 10,
    "龍": 9,
    "開": 9,
    "笛": 9,
    "立": 9,
    "百": 9,
    "白": 9,
    "獣": 9,
    "月": 9,
    "己": 9,
    "唯": 9,
    "古": 9,
    "高": 8,
    "錘": 8,
    "謎": 8,
    "紙": 8,
    "皇": 8,
    "猫": 8,
    "満": 8,
    "止": 8,
    "挽": 8,
    "待": 8,
    "壁": 8,
    "四": 8,
    "名": 8,
    "反": 8,
    "傷": 8,
    "位": 8,
    "軸": 7,
    "船": 7,
    "米": 7,
    "混": 7,
    "毎": 7,
    "散": 7,
    "常": 7,
    "官": 7,
    "守": 7,
    "塩": 7,
    "倒": 7,
    "亦": 7,
    "馬": 6,
    "震": 6,
    "閉": 6,
    "論": 6,
    "裁": 6,
    "端": 6,
    "直": 6,
    "族": 6,
    "或": 6,
    "引": 6,
    "島": 6,
    "圧": 6,
    "叮": 6,
    "包": 6,
    "兵": 6,
    "黒": 5,
    "類": 5,
    "静": 5,
    "虫": 5,
    "牛": 5,
    "毛": 5,
    "撃": 5,
    "意": 5,
    "平": 5,
    "寒": 5,
    "墨": 5,
    "光": 5,
    "金": 4,
    "遠": 4,
    "煙": 4,
    "歌": 4,
    "層": 4,
    "季": 4,
    "嗅": 4,
    "哇": 4,
    "味": 4,
    "右": 4,
    "卵": 4,
    "万": 4,
    "㕮": 4,
    "骨": 3,
    "連": 3,
    "赤": 3,
    "謝": 3,
    "試": 3,
    "西": 3,
    "羊": 3,
    "穴": 3,
    "積": 3,
    "王": 3,
    "橋": 3,
    "棚": 3,
    "従": 3,
    "左": 3,
    "叫": 3,
    "凹": 3,
    "冬": 3,
    "六": 3,
    "青": 2,
    "躍": 2,
    "認": 2,
    "虎": 2,
    "肉": 2,
    "網": 2,
    "筒": 2,
    "笑": 2,
    "秋": 2,
    "祭": 2,
    "油": 2,
    "汪": 2,
    "檸": 2,
    "東": 2,
    "普": 2,
    "御": 2,
    "後": 2,
    "弓": 2,
    "広": 2,
    "山": 2,
    "咍": 2,
    "南": 2,
    "前": 2,
    "八": 2,
    "傾": 2,
    "五": 2,
    "鼓": 1,
    "迷": 1,
    "輪": 1,
    "軽": 1,
    "貝": 1,
    "豆": 1,
    "覆": 1,
    "血": 1,
    "膠": 1,
    "翰": 1,
    "羅": 1,
    "綿": 1,
    "糸": 1,
    "祖": 1,
    "硫": 1,
    "短": 1,
    "球": 1,
    "民": 1,
    "歪": 1,
    "正": 1,
    "樽": 1,
    "文": 1,
    "形": 1,
    "哦": 1,
    "北": 1,
    "俐": 1,
    "九": 1,
    "七": 1,
}
