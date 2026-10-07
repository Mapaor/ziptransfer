// @ts-nocheck
import nodemailer from 'nodemailer';
import { render } from '@react-email/render';
import TransferDownloadedEmail from './templates/TransferDownloadedEmail.jsx';
import TransferRequestReceivedEmail from './templates/TransferRequestReceivedEmail.jsx';
import TransferShareEmail from './templates/TransferShareEmail.jsx';
import TransferRequestShareEmail from './templates/TransferRequestShareEmail.jsx';
import PasswordResetEmail from './templates/PasswordResetEmail.jsx';
import MagicLinkEmail from './templates/MagicLinkEmail.jsx';

const tlsMode = process.env.SMTP_TLS?.toLowerCase() || 'starttls';

const transporter = process.env.SMTP_HOST ? nodemailer.createTransport({
  host: process.env.SMTP_HOST,
  port: parseInt(process.env.SMTP_PORT || '587'),
  secure: tlsMode === 'tls', // true for implicit TLS (usually port 465)
  requireTLS: tlsMode === 'starttls', // true to force STARTTLS upgrade
  ignoreTLS: tlsMode === 'none', // true to disable TLS completely
  auth: {
    user: process.env.SMTP_USER,
    pass: process.env.SMTP_PASS,
  },
}) : null;

async function sendMail(reactElement, { from, to, subject }) {
  const html = await render(reactElement);
  const sender = from || process.env.SMTP_FROM || "noreply@transfer.zip";

  if (transporter) {
    await transporter.sendMail({
      from: sender,
      to,
      subject,
      html,
    });
  } else {
    console.log('[MOCK] Sending email to', to, 'from', sender, 'subject', subject);
    console.log(html);
  }
}

export async function sendTransferDownloaded(email, { name, link, brand }) {
  await sendMail(TransferDownloadedEmail({ name, link, brand }), {
    to: email,
    subject: "Transfer downloaded - " + (brand?.name || process.env.NEXT_PUBLIC_SITE_NAME),
  });
}

export async function sendTransferRequestReceived(email, { name, link, brand }) {
  await sendMail(TransferRequestReceivedEmail({ name, link, brand }), {
    to: email,
    subject: "Files received - " + (brand?.name || process.env.NEXT_PUBLIC_SITE_NAME),
  });
}

export async function sendTransferRequestShare(email, { name, description, link, brand }) {
  await sendMail(TransferRequestShareEmail({ name, description, link, brand }), {
    to: email,
    subject: "Transfer request - " + (brand?.name || process.env.NEXT_PUBLIC_SITE_NAME),
  });
}

export async function sendTransferShare(email, { name, description, link, brand }) {
  await sendMail(TransferShareEmail({ name, description, link, brand }), {
    to: email,
    subject: "Files available - " + (brand?.name || process.env.NEXT_PUBLIC_SITE_NAME),
  });
}

export async function sendPasswordReset(email, { link }) {
  await sendMail(PasswordResetEmail({ link }), {
    to: email,
    subject: "Reset your password - " + process.env.NEXT_PUBLIC_SITE_NAME,
  });
}

export async function sendMagicLink(email, { link }) {
  await sendMail(MagicLinkEmail({ link }), {
    to: email,
    subject: "Log In - " + process.env.NEXT_PUBLIC_SITE_NAME,
  });
}
