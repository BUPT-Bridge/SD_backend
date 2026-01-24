from flask import Flask, request, jsonify
import random
import string
import logging

app = Flask(__name__)


def generate_random_string(length=20):
    """生成指定长度的随机字符串"""
    characters = string.ascii_letters + string.digits
    return "".join(random.choice(characters) for _ in range(length))


@app.route("/sns/jscode2session", methods=["GET"])
def jscode2session():
    # 获取URL参数
    appid = request.args.get("appid", "")
    secret = request.args.get("secret", "")
    js_code = request.args.get("js_code", "")
    grant_type = request.args.get("grant_type", "")

    # 在控制台打印参数
    print(
        f"收到请求 - appid: {appid}, secret: {secret}, js_code: {js_code}, grant_type: {grant_type}"
    )

    # 生成随机的session_key和openid
    session_key = generate_random_string(20)
    openid = generate_random_string(20)

    # 返回JSON响应
    response = {"session_key": session_key, "openid": js_code}

    return jsonify(response)


if __name__ == "__main__":
    # 配置日志，使控制台输出更清晰
    logging.basicConfig(level=logging.INFO)

    print("服务器启动中...")
    print("监听端口: 5000")
    print("访问地址: http://localhost:5000/sns/jscode2session")
    print(
        "示例: http://localhost:5000/sns/jscode2session?appid=wx123456&secret=abcdef&js_code=testcode123&grant_type=authorization_code"
    )
    print("按 Ctrl+C 停止服务器\n")

    # 启动Flask应用，监听所有IP地址的5000端口
    app.run(host="0.0.0.0", port=5000, debug=False)
