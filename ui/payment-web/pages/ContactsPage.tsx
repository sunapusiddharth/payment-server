// src/pages/ContactsPage.tsx
import React, { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAuth } from '../contexts/AuthContext';
import { ArrowLeftIcon, PhoneIcon } from '@heroicons/react/24/outline';

interface Contact {
  user_id: string;
  name: string;
  mobile: string;
}

const ContactsPage: React.FC = () => {
  const navigate = useNavigate();
  const { user } = useAuth();
  const [contacts, setContacts] = useState<Contact[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchContacts = async () => {
      try {
        const res = await fetch('/contacts', {
          headers: { Authorization: `Bearer ${localStorage.getItem('access_token')}` },
        });
        const data = await res.json();
        setContacts(data);
      } catch (error) {
        console.error(error);
      } finally {
        setLoading(false);
      }
    };
    fetchContacts();
  }, []);

  return (
    <div className="p-6 pt-12">
      <div className="flex items-center mb-6">
        <button onClick={() => navigate(-1)} className="mr-3">
          <ArrowLeftIcon className="w-6 h-6 text-gray-600" />
        </button>
        <h2 className="text-2xl font-bold">Contacts</h2>
      </div>

      {loading ? (
        <div className="space-y-4">
          {[1, 2, 3].map(i => (
            <div key={i} className="p-4 bg-white rounded-xl shadow border animate-pulse">
              <div className="h-4 bg-gray-200 rounded w-1/3 mb-2"></div>
              <div className="h-3 bg-gray-200 rounded w-1/2"></div>
            </div>
          ))}
        </div>
      ) : (
        <div className="space-y-4">
          {contacts.map((contact) => (
            <button
              key={contact.user_id}
              onClick={() => navigate('/send/phone', { state: { contact } })}
              className="w-full p-4 bg-white rounded-xl shadow text-left border flex items-center"
            >
              <div className="w-10 h-10 bg-primary text-white rounded-full flex items-center justify-center font-bold mr-3">
                {contact.name[0]}
              </div>
              <div>
                <p className="font-medium">{contact.name}</p>
                <p className="text-gray-600 text-sm">{contact.mobile}</p>
              </div>
              <PhoneIcon className="w-5 h-5 text-gray-400 ml-auto" />
            </button>
          ))}
        </div>
      )}
    </div>
  );
};

export default ContactsPage;