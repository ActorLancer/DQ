import Link from 'next/link'

export default function Footer() {
  return (
    <footer className="bg-primary-900 text-white mt-24">
      <div className="container-custom py-16">
        <div className="grid grid-cols-1 md:grid-cols-4 gap-8">
          {/* 关于 */}
          <div>
            <h3 className="text-lg font-bold mb-4">关于平台</h3>
            <ul className="space-y-2">
              <li>
                <Link href="/about" className="text-gray-300 hover:text-white transition-colors">
                  平台介绍
                </Link>
              </li>
              <li>
                <Link href="/trust-center" className="text-gray-300 hover:text-white transition-colors">
                  可信能力
                </Link>
              </li>
              <li>
                <Link href="/compliance" className="text-gray-300 hover:text-white transition-colors">
                  合规说明
                </Link>
              </li>
            </ul>
          </div>

          {/* 服务 */}
          <div>
            <h3 className="text-lg font-bold mb-4">服务</h3>
            <ul className="space-y-2">
              <li>
                <Link href="/marketplace" className="text-gray-300 hover:text-white transition-colors">
                  数据市场
                </Link>
              </li>
              <li>
                <Link href="/suppliers" className="text-gray-300 hover:text-white transition-colors">
                  优质供应商
                </Link>
              </li>
              <li>
                <Link href="/standard-flow" className="text-gray-300 hover:text-white transition-colors">
                  标准链路
                </Link>
              </li>
            </ul>
          </div>

          {/* 支持 */}
          <div>
            <h3 className="text-lg font-bold mb-4">支持</h3>
            <ul className="space-y-2">
              <li>
                <Link href="/docs" className="text-gray-300 hover:text-white transition-colors">
                  帮助文档
                </Link>
              </li>
              <li>
                <Link href="/api-docs" className="text-gray-300 hover:text-white transition-colors">
                  API 文档
                </Link>
              </li>
              <li>
                <Link href="/contact" className="text-gray-300 hover:text-white transition-colors">
                  联系我们
                </Link>
              </li>
            </ul>
          </div>

          {/* 法律 */}
          <div>
            <h3 className="text-lg font-bold mb-4">法律</h3>
            <ul className="space-y-2">
              <li>
                <Link href="/terms" className="text-gray-300 hover:text-white transition-colors">
                  服务条款
                </Link>
              </li>
              <li>
                <Link href="/privacy" className="text-gray-300 hover:text-white transition-colors">
                  隐私政策
                </Link>
              </li>
              <li>
                <Link href="/license" className="text-gray-300 hover:text-white transition-colors">
                  授权协议
                </Link>
              </li>
            </ul>
          </div>
        </div>

        <div className="border-t border-gray-700 mt-12 pt-8 text-center text-gray-400">
          <p>&copy; 2026 数据交易平台. 保留所有权利.</p>
        </div>
      </div>
    </footer>
  )
}
