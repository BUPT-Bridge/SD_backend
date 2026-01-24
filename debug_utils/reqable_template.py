import os
import re
import sys
import tempfile
import subprocess
from pathlib import Path
from typing import Dict
from google.protobuf import descriptor_pool, message_factory
from google.protobuf.json_format import MessageToDict, ParseDict

class ProtobufParser:
    def __init__(self, protobuf_schema: str):
        self.protobuf_schema = protobuf_schema
        self.tempdir = Path(tempfile.gettempdir()) / "reqable" / "protobuf"
        self.dynamiclib_dir = Path(tempfile.gettempdir()) / "reqable" / "dynamiclib"
        self.tempdir.mkdir(parents=True, exist_ok=True)
        self.dynamiclib_dir.mkdir(parents=True, exist_ok=True)

        # 添加动态库路径到 sys.path
        if str(self.dynamiclib_dir) not in sys.path:
            sys.path.insert(0, str(self.dynamiclib_dir))

        self._compiled_module = None
        self._message_classes = {}
        self._compile_proto()

    def _extract_package(self):
        """从 schema 中提取 package 名"""
        package_match = re.search(r'package\s+([\w.]+)\s*;', self.protobuf_schema)
        return package_match.group(1) if package_match else "default"

    def _compile_proto(self):
        """编译 proto 文件"""
        package_name = self._extract_package()

        # 写入 proto 文件
        proto_file = self.tempdir / f"{package_name.replace('.', '_')}.proto"
        proto_file.write_text(self.protobuf_schema, encoding='utf-8')

        # 使用 protoc 编译
        try:
            subprocess.run([
                'protoc',
                f'--proto_path={self.tempdir}',
                f'--python_out={self.dynamiclib_dir}',
                str(proto_file)
            ], check=True, capture_output=True)
        except subprocess.CalledProcessError as e:
            raise RuntimeError(f"protoc compilation failed: {e.stderr.decode()}")

        # 动态导入生成的模块
        module_name = f"{package_name.replace('.', '_')}_pb2"
        if module_name in sys.modules:
            del sys.modules[module_name]

        self._compiled_module = __import__(module_name)

        # 提取所有 message 类
        for attr_name in dir(self._compiled_module):
            attr = getattr(self._compiled_module, attr_name)
            if hasattr(attr, 'DESCRIPTOR') and hasattr(attr.DESCRIPTOR, 'full_name'):
                self._message_classes[attr_name] = attr

    def encode(self, message: dict, schema: str) -> bytes:
        """将字典序列化为 protobuf 字节

        Args:
            message: 要序列化的字典数据
            schema: message 类型名称
        """
        if schema not in self._message_classes:
            raise ValueError(f"Message type '{schema}' not found in proto schema")

        message_class = self._message_classes[schema]

        # 创建 message 实例
        msg = message_class()

        # 从字典填充 message
        ParseDict(message, msg)

        # 序列化
        return msg.SerializeToString()

    def decode(self, buffer: bytes, schema: str) -> dict:
        """将 protobuf 字节反序列化为字典

        Args:
            buffer: protobuf 字节数据
            schema: message 类型名称
        """
        if schema not in self._message_classes:
            raise ValueError(f"Message type '{schema}' not found in proto schema")

        message_class = self._message_classes[schema]

        # 创建 message 实例并解析
        msg = message_class()
        msg.ParseFromString(buffer)

        # 转换为字典
        return MessageToDict(msg, preserving_proto_field_name=True)


PROTO_STRING = r'''
syntax = "proto3";

package sd_backend.user;

// [Authorize::Admin && Authorize::User]
//
// User
// GET Payload = {
//  token: string
// }
//
// Admin
// GET Payload = {
//  token: string
//  ...
// }
message UserRequest {
  // [Authorize::Admin && Authorize::User]
  optional string target_openid = 10;
  optional string nickname = 2;
  optional string name = 3;
  optional string phone_number = 4;
  optional string address = 5;
  // optional string community = 6;
  // [Authorize::Admin]
  optional string is_important = 7;
  optional string avatar = 8;
  optional string permission = 9;
}

// [Authorize::User]
message User {
  // [Authorize::Admin && Authorize::User]
  optional string token = 1;
  optional string nickname = 2;
  optional string name = 3;
  optional string phone_number = 4;
  optional string address = 5;
  // optional string community = 6;
  // [Authorize::Admin]
  optional string is_important = 7;
  optional string avatar = 8;
  optional string permission = 9;
}

message UserResponse {
  optional User user = 1;
  int32 code = 2;
  string message = 3;
}

'''

# data = {
#     "target_openid": "dasdasd"
# }
# byte_data = parser.encode(data,"UserRequest")
# dict_data = parser.decode(byte_data,"UserRequest")
# print(dict_data)


from reqable import *
import json

parser = ProtobufParser(PROTO_STRING)
def onRequest(context, request):
    if str(request.body).strip() != "":
        json_data = json.loads(str(request.body))
        byte_data = parser.encode(json_data,"UserRequest")
        print(byte_data)
        request.body = byte_data
    return request

def onResponse(context, response):
    print(response.body)
    header_dict = response.headers.toDict()
    if not ("text/plain" in header_dict.get('content-type', '')):
        response.body = parser.decode(bytes(response.body),"UserResponse")
    return response
