const CHAR_COUNT = {
    "寒": 5,
    "己": 9,
    "祖": 1,
    "守": 7,
    "哦": 1,
    "震": 6,
    "閉": 6,
    "須": 48,
    "硬": 23,
    "上": 56,
    "神": 20,
    "処": 209,
    "龍": 9,
    "散": 7,
    "東": 2,
    "位": 8,
    "翰": 1,
    "清": 23,
    "国": 83,
    "机": 15,
    "色": 12,
    "奮": 12,
    "其": 32,
    "再": 32,
    "膠": 1,
    "皇": 8,
    "棚": 3,
    "集": 26,
    "右": 4,
    "雪": 16,
    "後": 2,
    "謎": 8,
    "連": 3,
    "無": 206,
    "静": 5,
    "直": 6,
    "衣": 22,
    "止": 8,
    "術": 23,
    "光": 5,
    "果": 27,
    "与": 97,
    "時": 102,
    "笛": 9,
    "歌": 4,
    "羊": 3,
    "筆": 45,
    "血": 1,
    "墨": 5,
    "三": 11,
    "御": 2,
    "多": 64,
    "遊": 18,
    "卵": 4,
    "木": 38,
    "或": 6,
    "壊": 15,
    "訴": 12,
    "周": 36,
    "哩": 45,
    "煙": 4,
    "席": 11,
    "裁": 6,
    "㕮": 4,
    "山": 2,
    "汪": 2,
    "左": 3,
    "塩": 6,
    "友": 48,
    "黒": 5,
    "如": 42,
    "嗅": 4,
    "試": 3,
    "六": 2,
    "汝": 248,
    "輪": 1,
    "手": 75,
    "言": 266,
    "米": 7,
    "門": 26,
    "哇": 4,
    "学": 174,
    "錘": 8,
    "馬": 6,
    "層": 4,
    "別": 31,
    "字": 21,
    "島": 6,
    "王": 3,
    "労": 72,
    "冬": 3,
    "白": 9,
    "蜜": 11,
    "覆": 1,
    "躍": 2,
    "包": 6,
    "端": 6,
    "付": 14,
    "何": 197,
    "豊": 10,
    "母": 16,
    "季": 3,
    "従": 3,
    "羅": 1,
    "硫": 1,
    "夏": 18,
    "網": 2,
    "力": 69,
    "俐": 1,
    "立": 9,
    "故": 23,
    "此": 336,
    "反": 8,
    "二": 31,
    "値": 13,
    "牌": 42,
    "八": 2,
    "勿": 24,
    "家": 91,
    "識": 116,
    "星": 43,
    "待": 8,
    "声": 17,
    "絵": 31,
    "足": 25,
    "論": 6,
    "古": 9,
    "七": 1,
    "全": 34,
    "定": 10,
    "平": 5,
    "形": 1,
    "為": 42,
    "倉": 17,
    "小": 29,
    "戦": 18,
    "樽": 1,
    "加": 37,
    "道": 38,
    "我": 513,
    "檸": 2,
    "金": 4,
    "橋": 3,
    "車": 29,
    "琴": 10,
    "骨": 3,
    "牛": 4,
    "豆": 1,
    "糸": 1,
    "引": 6,
    "助": 18,
    "謝": 3,
    "律": 20,
    "即": 18,
    "歪": 1,
    "万": 4,
    "乎": 114,
    "火": 31,
    "正": 1,
    "耳": 21,
    "九": 1,
    "撃": 5,
    "認": 2,
    "満": 8,
    "凹": 3,
    "同": 22,
    "筒": 2,
    "貧": 10,
    "花": 14,
    "水": 66,
    "倒": 7,
    "噫": 51,
    "鼓": 1,
    "魚": 14,
    "兵": 6,
    "始": 16,
    "女": 27,
    "民": 1,
    "西": 3,
    "終": 172,
    "行": 300,
    "一": 91,
    "体": 10,
    "心": 252,
    "子": 48,
    "秋": 2,
    "父": 21,
    "等": 130,
    "迷": 1,
    "種": 20,
    "将": 10,
    "大": 37,
    "類": 5,
    "受": 22,
    "前": 2,
    "猫": 8,
    "片": 16,
    "囲": 12,
    "口": 68,
    "族": 6,
    "咍": 1,
    "急": 15,
    "獣": 9,
    "来": 95,
    "犬": 20,
    "銭": 140,
    "圧": 6,
    "書": 91,
    "少": 16,
    "穴": 3,
    "綿": 1,
    "百": 9,
    "短": 1,
    "天": 42,
    "青": 2,
    "美": 11,
    "唯": 9,
    "叫": 3,
    "機": 39,
    "十": 16,
    "弓": 2,
    "冠": 34,
    "於": 331,
    "男": 34,
    "叮": 6,
    "肉": 2,
    "南": 1,
    "刀": 19,
    "石": 13,
    "茶": 33,
    "闇": 23,
    "物": 18,
    "虫": 5,
    "遠": 4,
    "甘": 26,
    "意": 4,
    "地": 21,
    "箱": 12,
    "件": 25,
    "之": 360,
    "生": 59,
    "悪": 14,
    "月": 9,
    "享": 14,
    "官": 7,
    "北": 1,
    "善": 174,
    "広": 2,
    "入": 28,
    "油": 2,
    "祭": 2,
    "川": 18,
    "彼": 432,
    "而": 61,
    "鳥": 13,
    "草": 14,
    "紙": 9,
    "真": 24,
    "下": 80,
    "挽": 8,
    "毛": 5,
    "人": 139,
    "風": 43,
    "常": 7,
    "互": 26,
    "長": 18,
    "使": 51,
    "貝": 1,
    "壁": 8,
    "杯": 16,
    "開": 9,
    "輩": 23,
    "軟": 12,
    "積": 3,
    "春": 11,
    "球": 1,
    "目": 69,
    "寝": 22,
    "普": 2,
    "淮": 19,
    "亦": 7,
    "毎": 7,
    "名": 8,
    "酒": 14,
    "四": 8,
    "五": 2,
    "新": 11,
    "味": 4,
    "軽": 1,
    "混": 7,
    "高": 8,
    "虎": 2,
    "日": 86,
    "激": 17,
    "笑": 2,
    "文": 1,
    "怖": 11,
    "傷": 8,
    "在": 312,
    "傾": 2,
    "極": 175,
    "軸": 7,
    "失": 11,
    "船": 7,
    "赤": 3,
}
