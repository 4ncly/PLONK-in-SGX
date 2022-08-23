import rsa
# 先生成一对密钥，然后保存.pem格式文件，当然也可以直接使用
(pubkey, privkey) = rsa.newkeys(3072,exponent=3)


pri = privkey.save_pkcs1().decode()
prifile = open('private.pem','w+')
prifile.write(pri)
prifile.close()