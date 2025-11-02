import { QRCodeSVG } from 'qrcode.react';

function AuthView({ authUrl }) {
  return (
    <div className="text-center py-8">
      <p className="text-lg text-gray-700 mb-6">
        Scan this QR code with your Pubky app to login:
      </p>

      <div className="flex justify-center mb-6">
        <div className="bg-white p-4 rounded-lg shadow-md">
          <QRCodeSVG
            value={authUrl}
            size={256}
            level="M"
            includeMargin={true}
          />
        </div>
      </div>

      <p className="text-gray-600 italic mb-2">Waiting for authentication...</p>
      <div className="flex justify-center">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
      </div>
    </div>
  );
}

export default AuthView;
